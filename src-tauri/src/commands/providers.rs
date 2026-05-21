//! IPC: 第三方 LLM 中转 API 集中管理（Provider Profiles）
//!
//! 设计：[`docs/proposals/agent-config-autoapply.md`]
//! Phase 1（W6）：Profile CRUD + Keychain 凭据 + 虚拟 key 发放
//! Phase 2（W7）：ConfigProbe + 一键覆盖 + snapshot 回滚

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// Keychain SERVICE 前缀（参考 storage/keychain.rs）
const CRED_KEY_PREFIX: &str = "provider.";

// ──────────────────────────────────────────────────────────────────
// DTO
// ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct ProviderProfileDto {
    pub id: String,
    pub name: String,
    pub provider_kind: String,
    pub protocol: String,
    pub base_url: String,
    pub default_model: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub virtual_key: String,
    pub is_default: bool,
    pub enabled: bool,
    pub credential_set: bool, // 是否已在 Keychain 存了 key
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateProfileInput {
    pub name: String,
    pub provider_kind: String,
    pub protocol: String,
    pub base_url: String,
    pub default_model: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub api_key: Option<String>, // 可选；为空则后续单独 set_provider_credential
    pub is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateProfilePatch {
    pub name: String,
    pub provider_kind: String,
    pub protocol: String,
    pub base_url: String,
    pub default_model: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub enabled: bool,
}

#[derive(Serialize, Clone)]
pub struct ConnTestResult {
    pub ok: bool,
    pub latency_ms: Option<u32>,
    pub status_code: Option<u16>,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ActiveRoutingDto {
    pub agent_id: String,
    pub profile_id: String,
    pub virtual_key: String,
    pub updated_at: String,
}

/// 一个可导入的中转候选——可能来自单个或多个 Agent（同 base_url+key 合并）
#[derive(Serialize, Clone)]
pub struct ImportCandidate {
    /// 哈希形成的稳定 id（agent_ids 排序后 + base_url 短哈希）
    pub candidate_id: String,
    pub suggested_name: String,
    pub inferred_kind: String,
    pub inferred_protocol: String,
    pub base_url: String,
    /// 仅展示用："sk-or-***xyz"
    pub api_key_masked: String,
    pub source_agents: Vec<String>,
    pub source_labels: Vec<String>,
    /// 是否与已有 Profile base_url 冲突
    pub conflicts_with_existing_profile: bool,
    pub existing_profile_name: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkImportInput {
    pub candidate_ids: Vec<String>,
    pub set_first_as_default: Option<bool>,
}

#[derive(Serialize)]
pub struct BulkImportResult {
    pub created: Vec<ProviderProfileDto>,
    pub skipped: Vec<SkippedCandidate>,
}

#[derive(Serialize)]
pub struct SkippedCandidate {
    pub candidate_id: String,
    pub reason: String,
}

// ──────────────────────────────────────────────────────────────────
// 内部 helpers
// ──────────────────────────────────────────────────────────────────

fn uuid_like() -> String {
    // 简化 uuid v7（时间序）：13 位毫秒时间戳 + 12 位随机十六进制
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let rand = rand_hex(12);
    format!("{:013x}-{}", ms, rand)
}

fn rand_hex(n: usize) -> String {
    // 不引入新 crate：用 nanos + ptr 地址简单熵；W6+ 切换 ed25519-dalek 内置 OsRng
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut s = String::with_capacity(n);
    let mut x = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0xdeadbeef);
    x ^= (&s as *const _ as usize as u64).rotate_left(13);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let nibble = ((x >> 32) & 0xF) as u8;
        s.push(if nibble < 10 {
            (b'0' + nibble) as char
        } else {
            (b'a' + nibble - 10) as char
        });
    }
    s
}

fn new_virtual_key() -> String {
    format!("sk-claw-{}", rand_hex(32))
}

fn cred_key(profile_id: &str) -> String {
    format!("{}{}", CRED_KEY_PREFIX, profile_id)
}

#[cfg(feature = "storage")]
fn row_to_dto(
    row: crate::storage::queries::providers::ProviderProfileRow,
    credential_set: bool,
) -> ProviderProfileDto {
    let headers = row
        .headers_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
    ProviderProfileDto {
        id: row.id,
        name: row.name,
        provider_kind: row.provider_kind,
        protocol: row.protocol,
        base_url: row.base_url,
        default_model: row.default_model,
        headers,
        virtual_key: row.virtual_key,
        is_default: row.is_default,
        enabled: row.enabled,
        credential_set,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

const ALLOWED_PROTOCOLS: &[&str] = &[
    "openai",
    "anthropic",
    "gemini",
    "ollama",
    "openai_responses",
];

fn validate_profile_input(
    name: &str,
    protocol: &str,
    base_url: &str,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Profile 名称不能为空".into());
    }
    if !ALLOWED_PROTOCOLS.contains(&protocol) {
        return Err(format!(
            "未知协议 {}，允许：{:?}",
            protocol, ALLOWED_PROTOCOLS
        ));
    }
    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err("base_url 必须以 http:// 或 https:// 开头".into());
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────
// Profile CRUD
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_provider_profiles(
    state: State<AppState>,
) -> AppResult<Vec<ProviderProfileDto>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let rows = crate::storage::queries::providers::list(db)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            let dtos: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    let cred_present = crate::storage::keychain::fetch(&cred_key(&r.id))
                        .ok()
                        .flatten()
                        .is_some();
                    row_to_dto(r, cred_present)
                })
                .collect();
            return Ok(dtos);
        }
    }
    let _ = state;
    Ok(vec![])
}

fn create_profile_inner(
    state: &State<AppState>,
    input: CreateProfileInput,
) -> AppResult<ProviderProfileDto> {
    validate_profile_input(&input.name, &input.protocol, &input.base_url)
        .map_err(AppError::Other)?;

    let id = uuid_like();
    let virtual_key = new_virtual_key();
    let credential_ref = cred_key(&id);
    let headers_json = input
        .headers
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into()));
    let make_default = input.is_default.unwrap_or(false);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            if make_default {
                crate::storage::queries::providers::clear_default(db)
                    .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            }
            let auto_default = make_default
                || crate::storage::queries::providers::count_default(db).unwrap_or(0) == 0;

            crate::storage::queries::providers::insert(
                db,
                &id,
                &input.name,
                &input.provider_kind,
                &input.protocol,
                &input.base_url,
                &credential_ref,
                input.default_model.as_deref(),
                headers_json.as_deref(),
                &virtual_key,
                auto_default,
                true,
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;

            if let Some(api_key) = input.api_key.as_ref() {
                if !api_key.is_empty() {
                    crate::storage::keychain::store(&credential_ref, api_key)
                        .map_err(AppError::Other)?;
                }
            }

            let row = crate::storage::queries::providers::get(db, &id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .ok_or_else(|| AppError::Other("Provider 创建后未找到".into()))?;
            let cred_present = crate::storage::keychain::fetch(&credential_ref)
                .ok()
                .flatten()
                .is_some();
            tracing::info!(profile_id = %id, "provider profile created");
            return Ok(row_to_dto(row, cred_present));
        }
    }

    let _ = (state, headers_json, make_default);
    Err(AppError::NotImplemented("storage feature disabled"))
}

#[tauri::command]
pub fn create_provider_profile(
    state: State<AppState>,
    input: CreateProfileInput,
) -> AppResult<ProviderProfileDto> {
    create_profile_inner(&state, input)
}

#[tauri::command]
pub fn update_provider_profile(
    state: State<AppState>,
    id: String,
    patch: UpdateProfilePatch,
) -> AppResult<ProviderProfileDto> {
    validate_profile_input(&patch.name, &patch.protocol, &patch.base_url)
        .map_err(AppError::Other)?;
    let headers_json = patch
        .headers
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into()));

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            crate::storage::queries::providers::update(
                db,
                &id,
                &patch.name,
                &patch.provider_kind,
                &patch.protocol,
                &patch.base_url,
                patch.default_model.as_deref(),
                headers_json.as_deref(),
                patch.enabled,
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            let row = crate::storage::queries::providers::get(db, &id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .ok_or_else(|| AppError::Other("Profile 不存在".into()))?;
            let cred_present = crate::storage::keychain::fetch(&cred_key(&id))
                .ok()
                .flatten()
                .is_some();
            return Ok(row_to_dto(row, cred_present));
        }
    }
    let _ = (state, id, patch, headers_json);
    Err(AppError::NotImplemented("storage feature disabled"))
}

#[tauri::command]
pub fn delete_provider_profile(state: State<AppState>, id: String) -> AppResult<bool> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            // 删除凭据
            let _ = crate::storage::keychain::delete(&cred_key(&id));
            crate::storage::queries::providers::delete(db, &id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            tracing::info!(profile_id = %id, "provider profile deleted");
            return Ok(true);
        }
    }
    let _ = (state, id);
    Ok(false)
}

#[tauri::command]
pub fn set_default_provider_profile(
    state: State<AppState>,
    id: String,
) -> AppResult<ProviderProfileDto> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            crate::storage::queries::providers::clear_default(db)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            crate::storage::queries::providers::mark_default(db, &id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            let row = crate::storage::queries::providers::get(db, &id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .ok_or_else(|| AppError::Other("Profile 不存在".into()))?;
            let cred_present = crate::storage::keychain::fetch(&cred_key(&id))
                .ok()
                .flatten()
                .is_some();
            return Ok(row_to_dto(row, cred_present));
        }
    }
    let _ = (state, id);
    Err(AppError::NotImplemented("storage feature disabled"))
}

// ──────────────────────────────────────────────────────────────────
// 凭据（Keychain）
// ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_provider_credential(
    state: State<AppState>,
    profile_id: String,
    api_key: String,
) -> AppResult<bool> {
    if api_key.trim().is_empty() {
        return Err(AppError::Other("API key 不能为空".into()));
    }
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let exists = crate::storage::queries::providers::get(db, &profile_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .is_some();
            if !exists {
                return Err(AppError::Other("Profile 不存在".into()));
            }
            crate::storage::keychain::store(&cred_key(&profile_id), &api_key)
                .map_err(AppError::Other)?;
            tracing::info!(profile_id = %profile_id, "credential set");
            return Ok(true);
        }
    }
    let _ = (state, profile_id, api_key);
    Err(AppError::NotImplemented("storage feature disabled"))
}

#[tauri::command]
pub fn clear_provider_credential(
    state: State<AppState>,
    profile_id: String,
) -> AppResult<bool> {
    #[cfg(feature = "storage")]
    {
        let _ = &state;
        crate::storage::keychain::delete(&cred_key(&profile_id))
            .map_err(AppError::Other)?;
        return Ok(true);
    }
    #[allow(unreachable_code)]
    {
        let _ = (state, profile_id);
        Ok(false)
    }
}

#[tauri::command]
pub fn test_provider_connection(
    state: State<AppState>,
    profile_id: String,
) -> AppResult<ConnTestResult> {
    // W7 接通 reqwest 真实测试；当前阶段：仅检查 base_url 格式 + 凭据是否存在
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let row = crate::storage::queries::providers::get(db, &profile_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?
                .ok_or_else(|| AppError::Other("Profile 不存在".into()))?;
            let cred_present = crate::storage::keychain::fetch(&cred_key(&profile_id))
                .ok()
                .flatten()
                .is_some();
            if !cred_present {
                return Ok(ConnTestResult {
                    ok: false,
                    latency_ms: None,
                    status_code: None,
                    message: "未设置 API 凭据，无法测试连接".into(),
                });
            }
            return Ok(ConnTestResult {
                ok: true,
                latency_ms: Some(0),
                status_code: None,
                message: format!(
                    "base_url 与凭据已就绪；真实连接测试在 W7 接入 reqwest 后启用（{}）",
                    row.base_url
                ),
            });
        }
    }
    let _ = (state, profile_id);
    Ok(ConnTestResult {
        ok: false,
        latency_ms: None,
        status_code: None,
        message: "存储未启用".into(),
    })
}

// ──────────────────────────────────────────────────────────────────
// 运行时路由（W7 与 hudsucker 接通；当前 stub）
// ──────────────────────────────────────────────────────────────────

// ──────────────────────────────────────────────────────────────────
// 从已发现 Agent 导入候选（W6.5）
// ──────────────────────────────────────────────────────────────────

fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    let len = chars.len();
    if len <= 10 {
        return "***".into();
    }
    let head: String = chars.iter().take(6).collect();
    let tail: String = chars.iter().skip(len - 4).collect();
    format!("{}***{}", head, tail)
}

fn infer_kind(base_url: &str) -> &'static str {
    let lower = base_url.to_lowercase();
    if lower.contains("openrouter.ai") { return "openrouter"; }
    if lower.contains(".openai.azure.com") { return "azure"; }
    if lower.contains("api.openai.com") { return "openai"; }
    if lower.contains("api.anthropic.com") { return "anthropic"; }
    if lower.contains("deepbricks") { return "deepbricks"; }
    if lower.contains("generativelanguage.googleapis.com") { return "openai"; }
    if lower.contains("api.x.ai") { return "openai"; }
    if lower.starts_with("http://localhost")
        || lower.starts_with("http://127.")
        || lower.starts_with("https://localhost")
    {
        return "litellm";
    }
    "custom"
}

fn infer_protocol_from_kind(kind: &str) -> &'static str {
    match kind {
        "anthropic" => "anthropic",
        _ => "openai",
    }
}

fn short_hash(input: &str) -> String {
    // 简单 FNV-1a hash → 8 hex chars
    let mut h: u64 = 14695981039346656037;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    format!("{:016x}", h)[..8].to_string()
}

fn candidate_suggested_name(kind: &str, sources: &[String]) -> String {
    let kind_label = match kind {
        "openrouter" => "OpenRouter",
        "azure" => "Azure OpenAI",
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        "deepbricks" => "DeepBricks",
        "litellm" => "本地 LiteLLM",
        _ => "自定义中转",
    };
    if sources.len() == 1 {
        format!("{} ({})", kind_label, sources[0])
    } else if sources.len() > 1 {
        format!("{} (共享 {} 处)", kind_label, sources.len())
    } else {
        kind_label.into()
    }
}

/// 内部结构：单个 Agent 读到的凭据 + 推断信息
struct AgentCredential {
    agent_id: String,
    agent_name: String,
    base_url: String,
    api_key: String,
    source_label: String,
}

fn scan_all_agent_credentials() -> Vec<AgentCredential> {
    let agents = crate::agents::scanner::Scanner::with_default_platforms().scan_once();
    let mut out = Vec::new();
    for agent in &agents {
        if let Some(probe) = crate::agents::config_probe::probe_for(&agent.platform) {
            if let Some(cred) = probe.inspect_with_credential(agent) {
                // 跳过 ClawHeart 自己的虚拟 key（避免循环导入）
                if cred.api_key.starts_with("sk-claw-") {
                    continue;
                }
                // 跳过本地代理端点
                if cred.base_url.contains("127.0.0.1") || cred.base_url.contains("localhost") {
                    if cred.api_key.starts_with("sk-claw-") {
                        continue;
                    }
                }
                out.push(AgentCredential {
                    agent_id: format!("{}/{}", agent.platform, agent.agent_name),
                    agent_name: agent.agent_name.clone(),
                    base_url: cred.base_url,
                    api_key: cred.api_key,
                    source_label: cred.source_label,
                });
            }
        }
    }
    out
}

#[tauri::command]
pub fn scan_import_candidates(state: State<AppState>) -> AppResult<Vec<ImportCandidate>> {
    let creds = scan_all_agent_credentials();
    // 按 (base_url, api_key) 合并 —— 同 base_url+key 算同一候选
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<&AgentCredential>> = BTreeMap::new();
    for c in &creds {
        let key = format!("{}|{}", c.base_url, c.api_key);
        groups.entry(key).or_default().push(c);
    }

    // 拿已有 Profile 列表用于冲突检测
    let mut existing_base_urls: Vec<(String, String)> = Vec::new();
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            if let Ok(rows) = crate::storage::queries::providers::list(db) {
                existing_base_urls = rows
                    .into_iter()
                    .map(|r| (r.base_url, r.name))
                    .collect();
            }
        }
    }
    let _ = &state;

    let mut out: Vec<ImportCandidate> = Vec::new();
    for (_k, items) in groups {
        let first = items[0];
        let kind = infer_kind(&first.base_url);
        let protocol = infer_protocol_from_kind(kind);
        let source_agents: Vec<String> =
            items.iter().map(|c| c.agent_id.clone()).collect();
        let source_labels: Vec<String> =
            items.iter().map(|c| c.source_label.clone()).collect();
        let suggested_name = candidate_suggested_name(
            kind,
            &items.iter().map(|c| c.agent_name.clone()).collect::<Vec<_>>(),
        );
        let conflict = existing_base_urls
            .iter()
            .find(|(b, _)| b.trim_end_matches('/') == first.base_url.trim_end_matches('/'))
            .map(|(_, n)| n.clone());
        let candidate_id = format!("imp_{}", short_hash(&format!("{}|{}", first.base_url, first.api_key)));
        out.push(ImportCandidate {
            candidate_id,
            suggested_name,
            inferred_kind: kind.to_string(),
            inferred_protocol: protocol.to_string(),
            base_url: first.base_url.clone(),
            api_key_masked: mask_key(&first.api_key),
            source_agents,
            source_labels,
            conflicts_with_existing_profile: conflict.is_some(),
            existing_profile_name: conflict,
        });
    }
    Ok(out)
}

/// 检测到但**未能自动导入**的 Agent —— 通常是 OAuth、环境变量、或 Probe 未实现凭据读取
#[derive(Serialize, Clone)]
pub struct UnmanagedAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_platform: String,
    /// 检测到的原因码：oauth | env_var | already_proxied | no_probe | no_config
    pub reason: String,
    /// UI 友好的解释
    pub reason_label: String,
    /// 推荐用户做什么
    pub hint: String,
}

#[tauri::command]
pub fn scan_unmanaged_agents() -> AppResult<Vec<UnmanagedAgent>> {
    let agents = crate::agents::scanner::Scanner::with_default_platforms().scan_once();
    let mut out = Vec::new();
    for agent in &agents {
        let agent_id = format!("{}/{}", agent.platform, agent.agent_name);
        let probe = crate::agents::config_probe::probe_for(&agent.platform);

        let (reason, reason_label, hint) = match probe {
            None => (
                "no_probe",
                "暂无 Probe 支持",
                "ClawHeart 尚未支持该 Agent 的凭据读取；可手动在「中转配置」新建 Profile，然后用「Agent 一键覆盖」注入",
            ),
            Some(p) => match p.inspect_with_credential(agent) {
                None => match agent.platform.as_str() {
                    "claude" => (
                        "oauth",
                        "OAuth 订阅登录",
                        "Claude Code 用 Anthropic 账号 OAuth 登录，凭据不在配置文件里。手动建 Profile + 用「一键覆盖」即可写入 settings.json 强制走 ClawHeart",
                    ),
                    "codex" => (
                        "env_var",
                        "环境变量",
                        "Codex 使用 OPENAI_API_KEY 环境变量。手动建 Profile + 用「一键覆盖」可在 auth.json / config.toml 中强制注入",
                    ),
                    _ => (
                        "no_config",
                        "未在配置中发现凭据",
                        "Agent 已被发现但凭据不在配置文件里。手动建 Profile + 用「一键覆盖」即可托管",
                    ),
                },
                Some(cred) => {
                    // 有读到但被过滤的：sk-claw-* 或本地端点 → 已是 ClawHeart 代理
                    if cred.api_key.starts_with("sk-claw-")
                        || cred.base_url.contains("127.0.0.1")
                        || cred.base_url.contains("localhost")
                    {
                        ("already_proxied", "已托管", "该 Agent 已指向 ClawHeart 本地代理，无需再次导入")
                    } else {
                        continue; // 正常候选 — 由 scan_import_candidates 处理
                    }
                }
            },
        };

        out.push(UnmanagedAgent {
            agent_id,
            agent_name: agent.agent_name.clone(),
            agent_platform: agent.platform.clone(),
            reason: reason.into(),
            reason_label: reason_label.into(),
            hint: hint.into(),
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn bulk_import_profiles(
    state: State<AppState>,
    input: BulkImportInput,
) -> AppResult<BulkImportResult> {
    // 重新扫描（不依赖前端状态，stateless）
    let creds = scan_all_agent_credentials();
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<&AgentCredential>> = BTreeMap::new();
    for c in &creds {
        let key = format!("{}|{}", c.base_url, c.api_key);
        groups.entry(key).or_default().push(c);
    }

    let mut created: Vec<ProviderProfileDto> = Vec::new();
    let mut skipped: Vec<SkippedCandidate> = Vec::new();
    let set_default = input.set_first_as_default.unwrap_or(false);
    let mut first_import = true;

    for cid in &input.candidate_ids {
        // 通过 candidate_id 反查（重新计算所有 hash 直到匹配）
        let mut found_pair: Option<(&AgentCredential, Vec<&AgentCredential>)> = None;
        for (_k, items) in &groups {
            let first = items[0];
            let cand_id_check = format!(
                "imp_{}",
                short_hash(&format!("{}|{}", first.base_url, first.api_key))
            );
            if cand_id_check == *cid {
                found_pair = Some((first, items.clone()));
                break;
            }
        }
        let (first, items) = match found_pair {
            Some(p) => p,
            None => {
                skipped.push(SkippedCandidate {
                    candidate_id: cid.clone(),
                    reason: "候选已过期或不存在（请重新扫描）".into(),
                });
                continue;
            }
        };

        let kind = infer_kind(&first.base_url);
        let protocol = infer_protocol_from_kind(kind);
        let source_names: Vec<String> =
            items.iter().map(|c| c.agent_name.clone()).collect();
        let name = candidate_suggested_name(kind, &source_names);
        let make_default = set_default && first_import;

        let create_result = create_profile_inner(
            &state,
            CreateProfileInput {
                name,
                provider_kind: kind.into(),
                protocol: protocol.into(),
                base_url: first.base_url.clone(),
                default_model: None,
                headers: None,
                api_key: Some(first.api_key.clone()),
                is_default: Some(make_default),
            },
        );
        match create_result {
            Ok(dto) => {
                created.push(dto);
                first_import = false;
            }
            Err(e) => {
                skipped.push(SkippedCandidate {
                    candidate_id: cid.clone(),
                    reason: format!("创建失败：{}", e),
                });
            }
        }
    }

    tracing::info!(
        created = created.len(),
        skipped = skipped.len(),
        "bulk import completed"
    );

    Ok(BulkImportResult { created, skipped })
}

#[tauri::command]
pub fn list_active_routings(_state: State<AppState>) -> AppResult<Vec<ActiveRoutingDto>> {
    // W7 实现 active_routings 表的真实查询
    Ok(vec![])
}

#[tauri::command]
pub fn set_agent_routing(
    _state: State<AppState>,
    _agent_id: String,
    _profile_id: Option<String>,
) -> AppResult<Option<ActiveRoutingDto>> {
    // W7 实现
    Ok(None)
}
