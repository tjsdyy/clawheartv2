//! IPC: 监控模式（Access Tier）
//!
//! 三种监控模式：
//!   tier1 「端点映射」  — 反向代理（fetch_server）；应用层端点重写，无需 CA
//!   tier2 「系统代理」  — 正向代理（hudsucker MITM）；系统级 + 自签 CA
//!   tier3 「沙箱隔离」  — sandbox 子命令；OS 内核强制约束进程出口
//!
//! W4 基座绿灯前：所有真实切换为 stub；状态读写已通过 SQLite settings 持久化。
//! W5 hudsucker spike 后：set_access_mode("tier2") 真实启停 forward proxy。
//! W12 fetch_server 上线后：set_access_mode("tier1") 真实启停 :19112。
//! W20 sandbox 上线后：generate_sandbox_command 返回真实可执行命令。

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

const DEFAULT_FORWARD_PROXY_PORT: u16 = 19111;
const DEFAULT_REVERSE_PROXY_PORT: u16 = 19112;
const CA_PATH: &str = "~/.clawheart-v2/ca/clawheart-ca.pem";

const TIERS: &[&str] = &["tier1", "tier2", "tier3"];

/// 内置协议适配器清单（tier1 反向代理兼容路径）
const PROTOCOL_ADAPTERS: &[(&str, &str, &str)] = &[
    ("openai_chat",      "/v1/chat/completions", "OpenAI Chat Completions"),
    ("anthropic",        "/v1/messages",         "Anthropic Messages"),
    ("openai_responses", "/v1/responses",        "OpenAI Responses (Codex)"),
    ("gemini",           "/v1beta/models/:generateContent", "Google Gemini"),
    ("ollama",           "/api/chat",            "Ollama"),
];

/// 默认全部开启
const DEFAULT_ENABLED_ADAPTERS: &[&str] = &[
    "openai_chat",
    "anthropic",
    "openai_responses",
    "gemini",
    "ollama",
];

#[derive(Serialize, Deserialize, Clone)]
pub struct AccessModeInfo {
    pub current_tier: String,
    pub reverse_proxy_port: u16,
    pub forward_proxy_port: u16,
    pub ca_installed: bool,
    pub ca_path: String,
    pub system_proxy_active: bool,
    pub fetch_url_template: String,
    pub backend_ready: bool,
}

#[derive(Serialize, Clone)]
pub struct CaInstallResult {
    pub ok: bool,
    pub platform: String,
    pub message: String,
    pub manual_steps: Vec<String>,
    pub fingerprint: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct CaStatus {
    pub installed: bool,
    pub fingerprint: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct SandboxCommandPreview {
    pub command: String,
    pub platform: String,
    pub feature_available: bool,
    pub notes: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ProtocolAdapter {
    pub id: String,
    pub path: String,
    pub label: String,
    pub enabled: bool,
    pub default_enabled: bool,
}

#[derive(Serialize, Clone)]
pub struct PortUpdateResult {
    pub ok: bool,
    pub tier: String,
    pub port: u16,
    pub mode: AccessModeInfo,
}

#[derive(Serialize, Clone)]
pub struct FetchServerStatus {
    pub running: bool,
    pub port: u16,
    pub endpoint: String,
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

fn read_setting(state: &AppState, key: &str) -> Option<String> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            if let Ok(Some(v)) = crate::storage::queries::settings::get(db, key) {
                return Some(v);
            }
        }
    }
    let _ = (state, key);
    None
}

fn write_setting(state: &AppState, key: &str, value: &str) {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let _ = crate::storage::queries::settings::set(db, key, value);
        }
    }
    let _ = (state, key, value);
}

fn read_tier_from_db(state: &AppState) -> String {
    if let Some(v) = read_setting(state, "access_mode") {
        if TIERS.contains(&v.as_str()) {
            return v;
        }
    }
    "tier1".to_string()
}

fn read_port(state: &AppState, key: &str, default: u16) -> u16 {
    read_setting(state, key)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(default)
}

fn read_ca_installed(state: &AppState) -> bool {
    read_setting(state, "ca_installed")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

fn read_adapter_enabled(state: &AppState, id: &str) -> bool {
    let default = DEFAULT_ENABLED_ADAPTERS.contains(&id);
    read_setting(state, &format!("adapter_{}_enabled", id))
        .map(|v| v == "true" || v == "1")
        .unwrap_or(default)
}

/// 端口校验：1024-65535，避免冲突已知系统端口
fn validate_port(port: u16) -> Result<(), String> {
    if port < 1024 {
        return Err(format!(
            "端口 {} 在系统保留范围（< 1024）；请使用 1024-65535 之间的端口",
            port
        ));
    }
    // 常见冲突端口提醒
    const CONFLICT_PORTS: &[(u16, &str)] = &[
        (3000, "Node 开发服务器"),
        (5173, "Vite 默认端口"),
        (5432, "PostgreSQL"),
        (8080, "常用 Web 服务"),
        (8888, "pipelock 默认端口"),
    ];
    for (p, reason) in CONFLICT_PORTS {
        if *p == port {
            tracing::warn!(port, reason, "Possible port conflict");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_access_mode(state: State<AppState>) -> AppResult<AccessModeInfo> {
    use std::sync::atomic::Ordering;
    let tier = read_tier_from_db(&state);
    let ca_installed = read_ca_installed(&state);
    let reverse_port = read_port(&state, "reverse_proxy_port", DEFAULT_REVERSE_PROXY_PORT);
    let forward_port = read_port(&state, "forward_proxy_port", DEFAULT_FORWARD_PROXY_PORT);
    let actual_listen_port = state.fetch_server_port.load(Ordering::Acquire);
    let fetch_running = state.fetch_server_running.load(Ordering::Acquire);
    let proxy_running = state.proxy_server_running.load(Ordering::Acquire);
    // backend_ready：
    //   tier1 → fetch_server 起来即可
    //   tier2 → hudsucker 起来 + CA 已安装
    //   tier3 → sandbox 仅按需启动（无需常驻 server），backend_ready 始终为 false
    let backend_ready = match tier.as_str() {
        "tier1" => fetch_running,
        "tier2" => proxy_running && ca_installed,
        _ => false,
    };
    Ok(AccessModeInfo {
        current_tier: tier.clone(),
        reverse_proxy_port: if fetch_running { actual_listen_port } else { reverse_port },
        forward_proxy_port: forward_port,
        ca_installed,
        ca_path: CA_PATH.into(),
        system_proxy_active: tier == "tier2" && ca_installed,
        fetch_url_template: format!(
            "http://127.0.0.1:{}/v1",
            if fetch_running { actual_listen_port } else { reverse_port }
        ),
        backend_ready,
    })
}

#[tauri::command]
pub fn get_fetch_server_status(state: State<AppState>) -> AppResult<FetchServerStatus> {
    use std::sync::atomic::Ordering;
    let running = state.fetch_server_running.load(Ordering::Acquire);
    let port = state.fetch_server_port.load(Ordering::Acquire);
    Ok(FetchServerStatus {
        running,
        port,
        endpoint: format!("http://127.0.0.1:{}", port),
    })
}

#[tauri::command]
pub fn set_access_mode(state: State<AppState>, tier: String) -> AppResult<AccessModeInfo> {
    let normalized = if TIERS.contains(&tier.as_str()) {
        tier
    } else {
        "tier1".to_string()
    };
    write_setting(&state, "access_mode", &normalized);
    tracing::info!(tier = %normalized, "access mode changed");
    get_access_mode(state)
}

#[tauri::command]
pub fn update_proxy_port(
    state: State<AppState>,
    tier: String,
    port: u16,
) -> AppResult<PortUpdateResult> {
    validate_port(port).map_err(AppError::Other)?;

    let key = match tier.as_str() {
        "tier1" => "reverse_proxy_port",
        "tier2" => "forward_proxy_port",
        _ => return Err(AppError::Other(format!("tier {} 不支持端口配置", tier))),
    };
    write_setting(&state, key, &port.to_string());
    tracing::info!(tier = %tier, port, "proxy port updated");

    let mode = get_access_mode(state)?;
    Ok(PortUpdateResult {
        ok: true,
        tier,
        port,
        mode,
    })
}

#[tauri::command]
pub fn list_protocol_adapters(state: State<AppState>) -> AppResult<Vec<ProtocolAdapter>> {
    let adapters = PROTOCOL_ADAPTERS
        .iter()
        .map(|(id, path, label)| {
            let default = DEFAULT_ENABLED_ADAPTERS.contains(id);
            ProtocolAdapter {
                id: id.to_string(),
                path: path.to_string(),
                label: label.to_string(),
                enabled: read_adapter_enabled(&state, id),
                default_enabled: default,
            }
        })
        .collect();
    Ok(adapters)
}

#[tauri::command]
pub fn toggle_protocol_adapter(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> AppResult<Vec<ProtocolAdapter>> {
    if !PROTOCOL_ADAPTERS.iter().any(|(aid, _, _)| *aid == id) {
        return Err(AppError::Other(format!("未知协议适配器：{}", id)));
    }
    write_setting(
        &state.clone(),
        &format!("adapter_{}_enabled", id),
        if enabled { "true" } else { "false" },
    );
    tracing::info!(id = %id, enabled, "protocol adapter toggled");
    list_protocol_adapters(state)
}

#[tauri::command]
pub fn check_ca_status(state: State<AppState>) -> AppResult<CaStatus> {
    let installed = read_ca_installed(&state);
    Ok(CaStatus {
        installed,
        fingerprint: if installed {
            Some("sha256:abc123…（W6 真实指纹）".into())
        } else {
            None
        },
        expires_at: if installed {
            Some("2036-05-19".into())
        } else {
            None
        },
    })
}

#[tauri::command]
pub fn install_ca(state: State<AppState>) -> AppResult<CaInstallResult> {
    let platform = current_platform().to_string();

    // 1. 解析 CA 路径（绝对路径，需在 W5 ProxyServer 启动后生成完毕）
    let ca_path = match crate::state::data_dir().join("ca/clawheart-ca.pem").canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // CA 文件还不存在：返回 manual_steps 让用户先启用 proxy_real 再来
            let manual_steps = vec![
                "CA 证书尚未生成。请先切换到「系统代理」模式（监控模式工具页）".into(),
                "ClawHeart 启动后会自动在 ~/.clawheart-v2/ca/ 生成 CA".into(),
                "生成后再点此按钮即可一键安装".into(),
            ];
            return Ok(CaInstallResult {
                ok: false,
                platform,
                message: "CA 证书文件不存在，无法安装".into(),
                manual_steps,
                fingerprint: None,
            });
        }
    };
    let ca_path_str = ca_path.to_string_lossy().to_string();

    // 2. 平台分发
    let (auto_ok, message, manual_steps, fingerprint) = match platform.as_str() {
        "macos" => install_ca_macos(&ca_path_str),
        "linux" => install_ca_linux_stub(&ca_path_str),
        "windows" => install_ca_windows_stub(&ca_path_str),
        _ => (false, "未支持的平台".into(), vec![], None),
    };

    // 3. 自动安装成功才标记 ca_installed = true
    if auto_ok {
        write_setting(&state, "ca_installed", "true");
    }

    Ok(CaInstallResult {
        ok: auto_ok,
        platform,
        message,
        manual_steps,
        fingerprint,
    })
}

/// macOS 真实安装：用 osascript 调起原生密码框 → security add-trusted-cert
/// 成功率取决于 (a) 用户输密码正确 (b) 没装过同名 CA
fn install_ca_macos(ca_path: &str) -> (bool, String, Vec<String>, Option<String>) {
    use std::process::Command;

    // 构造 AppleScript 命令：弹原生 GUI 密码框
    let shell_cmd = format!(
        "security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain '{}'",
        ca_path.replace('\'', "'\\''")
    );
    let applescript = format!(
        "do shell script \"{}\" with administrator privileges",
        shell_cmd.replace('"', "\\\"")
    );

    tracing::info!(ca_path, "macOS: invoking security add-trusted-cert via osascript");
    let output = Command::new("osascript")
        .args(["-e", &applescript])
        .output();

    let manual_steps = vec![
        format!("打开「钥匙串访问」(Keychain Access)"),
        format!("文件 → 导入项 → 选择 {}", ca_path),
        format!("双击「ClawHeart Local CA」→ 信任 → 始终信任"),
        format!("或直接运行：sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain {}", ca_path),
    ];

    match output {
        Ok(out) if out.status.success() => (
            true,
            "CA 已成功安装到系统钥匙串".into(),
            manual_steps,
            Some(read_ca_fingerprint(ca_path).unwrap_or_default()),
        ),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(stderr = %stderr, "osascript exited non-zero");
            (
                false,
                if stderr.contains("User canceled") || stderr.contains("(-128)") {
                    "用户已取消密码弹窗".into()
                } else {
                    format!("自动安装失败：{}", stderr.trim())
                },
                manual_steps,
                None,
            )
        }
        Err(e) => (
            false,
            format!("无法调用 osascript：{}", e),
            manual_steps,
            None,
        ),
    }
}

/// Linux 真实安装：优先 pkexec 弹 PolicyKit 密码框 → install + update-ca-certificates
fn install_ca_linux_stub(ca_path: &str) -> (bool, String, Vec<String>, Option<String>) {
    use std::process::Command;

    let manual_steps = vec![
        format!(
            "sudo install -m 644 {} /usr/local/share/ca-certificates/clawheart.crt",
            ca_path
        ),
        "sudo update-ca-certificates".into(),
        "Firefox / Chrome 用户：在浏览器证书管理器中单独导入（独立 NSS DB）".into(),
    ];

    // 检查 pkexec 是否可用
    let has_pkexec = Command::new("which")
        .arg("pkexec")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_pkexec {
        return (
            false,
            "未检测到 pkexec（PolicyKit）；请按手动步骤操作".into(),
            manual_steps,
            None,
        );
    }

    let shell_cmd = format!(
        "install -m 644 '{}' /usr/local/share/ca-certificates/clawheart.crt && update-ca-certificates",
        ca_path.replace('\'', "'\\''")
    );

    tracing::info!(ca_path, "Linux: invoking pkexec sh -c install + update-ca-certificates");
    let output = Command::new("pkexec")
        .args(["sh", "-c", &shell_cmd])
        .output();

    match output {
        Ok(out) if out.status.success() => (
            true,
            "CA 已安装到系统受信任根证书".into(),
            manual_steps,
            Some(read_ca_fingerprint(ca_path).unwrap_or_default()),
        ),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(stderr = %stderr, "pkexec exited non-zero");
            (
                false,
                if stderr.contains("dismissed") || stderr.contains("cancelled") {
                    "用户已取消密码弹窗".into()
                } else {
                    format!("pkexec 失败：{}", stderr.trim())
                },
                manual_steps,
                None,
            )
        }
        Err(e) => (
            false,
            format!("无法调用 pkexec：{}", e),
            manual_steps,
            None,
        ),
    }
}

/// Windows 真实安装：PowerShell 调 Start-Process -Verb RunAs 弹 UAC → certutil
fn install_ca_windows_stub(ca_path: &str) -> (bool, String, Vec<String>, Option<String>) {
    use std::process::Command;

    let manual_steps = vec![
        "右键 CA 证书 → 安装证书 → 本地计算机".into(),
        "选择「将所有证书放入下列存储」→「受信任的根证书颁发机构」".into(),
        format!(
            "或以管理员 PowerShell 运行：certutil -addstore -f Root \"{}\"",
            ca_path
        ),
    ];

    // 用 PowerShell 调 Start-Process -Verb RunAs 触发 UAC 弹窗
    // 注意：certutil 命令参数用 ArgumentList 数组传，避免引号转义问题
    let ps_cmd = format!(
        "Start-Process -Wait -Verb RunAs -FilePath certutil -ArgumentList @('-addstore','-f','Root','{}')",
        ca_path.replace('\'', "''")
    );

    tracing::info!(ca_path, "Windows: invoking certutil via PowerShell + UAC");
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .output();

    match output {
        Ok(out) if out.status.success() => (
            true,
            "CA 已安装到受信任根证书颁发机构".into(),
            manual_steps,
            Some(read_ca_fingerprint(ca_path).unwrap_or_default()),
        ),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(stderr = %stderr, "powershell certutil exited non-zero");
            (
                false,
                if stderr.contains("operation was canceled by the user") {
                    "用户已取消 UAC 弹窗".into()
                } else {
                    format!("PowerShell 失败：{}", stderr.trim())
                },
                manual_steps,
                None,
            )
        }
        Err(e) => (
            false,
            format!("无法调用 PowerShell：{}", e),
            manual_steps,
            None,
        ),
    }
}

fn read_ca_fingerprint(ca_path: &str) -> Option<String> {
    let bytes = std::fs::read(ca_path).ok()?;
    let digest = crate::security::sha256::digest(&bytes);
    Some(format!("sha256:{}", crate::security::sha256::hex(&digest)))
}

#[tauri::command]
pub fn uninstall_ca(state: State<AppState>) -> AppResult<bool> {
    write_setting(&state, "ca_installed", "false");
    tracing::info!("CA marked as uninstalled");
    Ok(true)
}

#[tauri::command]
pub fn generate_sandbox_command(
    cmd: String,
    args: Vec<String>,
) -> AppResult<SandboxCommandPreview> {
    let platform = current_platform().to_string();
    let joined = if args.is_empty() {
        cmd.clone()
    } else {
        format!("{} {}", cmd, args.join(" "))
    };

    let (available, notes) = match platform.as_str() {
        "macos" => (
            false,
            vec![
                "W20 接入：基于 sandbox-exec + seatbelt profile".into(),
                "进程网络出口强制经由 ClawHeart，无旁路".into(),
            ],
        ),
        "linux" => (
            false,
            vec![
                "W20 接入：基于 Landlock + seccomp-bpf + namespace".into(),
                "需要内核版本 ≥ 5.13".into(),
            ],
        ),
        "windows" => (
            false,
            vec![
                "v2.1 接入：基于 AppContainer + WFP".into(),
                "GA 阶段 Windows 暂不支持沙箱模式".into(),
            ],
        ),
        _ => (false, vec!["未支持的平台".into()]),
    };

    Ok(SandboxCommandPreview {
        command: format!("clawheart sandbox -- {}", joined),
        platform,
        feature_available: available,
        notes,
    })
}
