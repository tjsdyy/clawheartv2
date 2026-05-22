//! 未知平台候选探测器
//!
//! 扫描 `~/.<name>/` 隐藏目录，识别可能的 AI Agent 候选。命中后返回
//! `status: "candidate"`，UI 引导用户确认是否纳入管理。
//!
//! 启发式信号（任一命中即列为候选）：
//! 1. 含 `skills/` 子目录
//! 2. 含 `mcp.json` / `.mcp.json` / `claude_desktop_config.json` 等已知 AI 配置文件
//! 3. 含任意 `*.json` / `*.yaml` / `*.yml` 顶层含 AI 相关 key（`api_key` / `apiKey` /
//!    `base_url` / `baseUrl` / `mcpServers` / `model` / `provider` 等）
//! 4. 含 `.env` 含 `*_API_KEY=...` 行
//!
//! 排除规则：
//! - 已知平台目录（claude/codex/cursor/gemini/windsurf/openclaw/openeva）
//! - 系统/工具链目录黑名单（.git/.cache/.npm/.cargo/.rustup/.ssh/.gnupg 等）
//! - 用户已在 settings 中标记 `ignore` 的目录（前端落库后扫描器再排除）

use super::PlatformScanner;
use crate::agents::DiscoveredAgent;
use std::path::Path;

pub struct UnknownPlatform;

/// 已知平台的目录名（不含前导点），用于在扫描中跳过。
const KNOWN_PLATFORM_DIRS: &[&str] = &[
    "claude", "codex", "cursor", "gemini", "windsurf",
    "openclaw", "openeva", "hermes",
];

/// 系统/工具链/常见非 AI 目录黑名单。
const SYSTEM_DIR_BLACKLIST: &[&str] = &[
    // ClawHeart 自身
    "clawheart", "clawheart-v2",
    // 版本管理 / 包管理 / 工具链
    "git", "cache", "npm", "cargo", "rustup", "ssh", "gnupg",
    "config", "local", "Trash", "vscode", "vscode-oss",
    "yarn", "pnpm", "nvm", "bun", "rbenv", "pyenv",
    "rye", "uv", "poetry", "deno",
    // 系统/编辑器/IDE
    "DS_Store", "Spotlight-V100", "fseventsd", "TemporaryItems",
    "android", "gradle", "m2", "ivy2", "sbt", "expo",
    "Xcode", "java", "p2",
    // 历史/缓存
    "node_repl_history", "lesshst", "viminfo", "zsh_history",
    "bash_history", "zcompdump", "bash_sessions", "CFUserTextEncoding",
    // 云/容器
    "docker", "docker-desktop", "kube", "minikube", "aws",
    "gcloud", "azure",
    // 其他工具
    "ipython", "jupyter", "matplotlib",
    "vim", "tmux", "fonts", "designer",
    "wakatime", "redhat", "Mozilla", "thunderbird",
    "subversion", "hg", "bzr",
    // 远程桌面 / 通信 / 输入法
    "anydesk", "bytertc", "sogouinput", "rustdesk", "teamviewer",
    // Electron 工具链
    "electron-gyp", "electron", "node-gyp",
];

/// AI 相关的 JSON/YAML 关键 key 名称。
const AI_HINT_KEYS: &[&str] = &[
    "api_key", "apiKey", "API_KEY",
    "base_url", "baseUrl", "BASE_URL",
    "mcpServers", "mcp_servers",
    "anthropic_api_key", "ANTHROPIC_API_KEY",
    "openai_api_key", "OPENAI_API_KEY",
    "gemini_api_key", "GEMINI_API_KEY",
    "model", "models", "provider", "providers",
];

impl PlatformScanner for UnknownPlatform {
    fn id(&self) -> &'static str { "unknown" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let Some(home) = super::dirs_home() else { return Ok(vec![]); };
        let mut results = Vec::new();

        let entries = match std::fs::read_dir(&home) {
            Ok(e) => e,
            Err(e) => return Err(format!("read_dir({}) failed: {}", home.display(), e)),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let Some(name_os) = path.file_name() else { continue; };
            let name = name_os.to_string_lossy();
            // 只看隐藏目录（以 . 开头）
            if !name.starts_with('.') { continue; }
            // 去掉前导点
            let bare = &name[1..];
            if bare.is_empty() { continue; }
            // 跳过已知平台
            if KNOWN_PLATFORM_DIRS.iter().any(|p| bare.eq_ignore_ascii_case(p)) {
                continue;
            }
            // 跳过系统黑名单
            if SYSTEM_DIR_BLACKLIST.iter().any(|p| bare.eq_ignore_ascii_case(p)) {
                continue;
            }

            let signals = collect_signals(&path);
            if signals.is_empty() { continue; }

            let display_name = pretty_name(bare);
            results.push(DiscoveredAgent {
                platform: format!("unknown:{}", bare),
                agent_name: display_name,
                config_path: Some(path.to_string_lossy().to_string()),
                process_name: None,
                last_seen: now_unix(),
                mcp_servers: extract_mcp_servers_from_signals(&path).unwrap_or_default(),
                config_hash: None,
                status: "candidate".into(),
                discovery_signals: signals,
            });
        }

        Ok(results)
    }
}

/// 在给定目录下收集 AI 相关线索；返回信号字符串列表（如 `["skills/", "mcp.json"]`）
fn collect_signals(dir: &Path) -> Vec<String> {
    let mut signals = Vec::new();

    // 信号 1: skills 子目录
    if dir.join("skills").is_dir() {
        signals.push("skills/".to_string());
    }

    // 信号 2: 已知 AI 配置文件名
    for fname in &[
        "mcp.json", ".mcp.json",
        "claude_desktop_config.json",
        "settings.json", "config.toml",
        "auth.json", "openai.json", "anthropic.json",
        "openrouter.json", "providers.json",
    ] {
        if dir.join(fname).is_file() {
            signals.push((*fname).to_string());
        }
    }

    // 信号 3: 顶层 .env 含 *_API_KEY
    if let Ok(content) = std::fs::read_to_string(dir.join(".env")) {
        if content.lines().any(|l| {
            let t = l.trim();
            !t.starts_with('#') && t.contains("_API_KEY=")
                && t.split('=').nth(1).map(|v| !v.trim().is_empty()).unwrap_or(false)
        }) {
            signals.push(".env:*_API_KEY".to_string());
        }
    }

    // 信号 4: 顶层 *.json / *.yaml / *.yml 含 AI 相关 key（仅扫描顶层，限 5 个文件，避免大目录卡顿）
    if signals.len() < 4 {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut scanned = 0;
            for entry in entries.flatten() {
                if scanned >= 5 { break; }
                let p = entry.path();
                if !p.is_file() { continue; }
                let Some(ext) = p.extension().and_then(|e| e.to_str()) else { continue; };
                if !matches!(ext, "json" | "yaml" | "yml") { continue; }
                // 文件 > 256KB 跳过（避免大日志文件）
                if std::fs::metadata(&p).map(|m| m.len() > 256 * 1024).unwrap_or(true) {
                    continue;
                }
                let Ok(content) = std::fs::read_to_string(&p) else { continue; };
                scanned += 1;
                if let Some(hit) = AI_HINT_KEYS.iter().find(|k| content.contains(*k)) {
                    let fname = p.file_name().map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "?.json".into());
                    signals.push(format!("{}:{}", fname, hit));
                    if signals.len() >= 4 { break; }
                }
            }
        }
    }

    signals
}

fn extract_mcp_servers_from_signals(dir: &Path) -> Option<Vec<String>> {
    for fname in &["mcp.json", ".mcp.json"] {
        let p = dir.join(fname);
        let Ok(content) = std::fs::read_to_string(&p) else { continue; };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { continue; };
        let servers = v
            .get("mcpServers")
            .or_else(|| v.get("servers"))
            .and_then(|s| s.as_object());
        if let Some(map) = servers {
            return Some(map.keys().cloned().collect());
        }
    }
    None
}

/// 将目录名美化为显示名："my-tool" → "My Tool"
fn pretty_name(bare: &str) -> String {
    bare.split(|c: char| c == '-' || c == '_' || c == '.')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn now_unix() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_name_works() {
        assert_eq!(pretty_name("my-tool"), "My Tool");
        assert_eq!(pretty_name("open_cara"), "Open Cara");
        assert_eq!(pretty_name("foo"), "Foo");
    }
}
