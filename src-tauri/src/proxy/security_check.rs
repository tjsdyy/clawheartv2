//! 代理路径的轻量安全检查 + 事件持久化（W13）
//!
//! 共用入口：fetch_server (axum :19112) 与 hudsucker handler (:19111) 都通过本模块：
//!   1. `run_security_check` 跑 KillSwitch + DLP + danger + injection
//!   2. `record_intercept` 写入 intercept_events
//!   3. `record_request_log` 写入 request_logs（W13.6）
//!   4. `record_token_usage` 写入 token_usage（W13.7）

use crate::security::{danger, injection, redact};
use crate::state::AppState;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum SecurityVerdict {
    Allow,
    /// 记录事件但继续转发（如 prompt injection 默认 warn）
    Warn(EventMeta),
    /// 阻断 + 自适应响应
    Block(EventMeta),
}

#[derive(Debug, Clone)]
pub struct EventMeta {
    pub event_type: &'static str,
    pub severity: &'static str,    // critical / high / medium / low
    pub signal_class: &'static str,
    pub rule_id: String,
    pub mitre_attack_id: Option<String>,
    pub reason: String,
}

impl SecurityVerdict {
    pub fn is_block(&self) -> bool {
        matches!(self, SecurityVerdict::Block(_))
    }
    pub fn meta(&self) -> Option<&EventMeta> {
        match self {
            SecurityVerdict::Allow => None,
            SecurityVerdict::Warn(m) | SecurityVerdict::Block(m) => Some(m),
        }
    }
}

/// 主入口：跑安全检查。优先级：KillSwitch > DLP > danger > injection。
pub fn run_security_check(app: &AppState, body_text: &str) -> SecurityVerdict {
    // L7：KillSwitch
    if app.kill_switch.snapshot() {
        return SecurityVerdict::Block(EventMeta {
            event_type: "kill_switch",
            severity: "critical",
            signal_class: "manual_block",
            rule_id: "KS-001".into(),
            mitre_attack_id: None,
            reason: "ClawHeart Kill Switch 已激活".into(),
        });
    }

    // L2.DLP（凭据外泄，最高优先级阻断）
    let dlp = redact::redact(body_text);
    if let Some(hit) = dlp.hits.first() {
        return SecurityVerdict::Block(EventMeta {
            event_type: "credential_leak",
            severity: "critical",
            signal_class: "credential_leak",
            rule_id: hit.pattern_id.clone(),
            mitre_attack_id: Some("T1552".into()),
            reason: format!("Credential leak: {}", hit.class),
        });
    }

    // L2.danger（危险指令）
    let dh = danger::scan(body_text);
    if let Some(hit) = dh.first() {
        return SecurityVerdict::Block(EventMeta {
            event_type: "danger_command",
            severity: "high",
            signal_class: "danger_command",
            rule_id: hit.rule_id.clone(),
            mitre_attack_id: hit.mitre_attack_id.clone(),
            reason: format!("危险指令命中：{}", hit.description),
        });
    }

    // L2.injection（默认 Warn，继续转发但记录）
    let inj = injection::scan(body_text);
    if let Some(hit) = inj.first() {
        return SecurityVerdict::Warn(EventMeta {
            event_type: "prompt_injection",
            severity: "medium",
            signal_class: "prompt_injection",
            rule_id: hit.pattern_id.clone(),
            mitre_attack_id: None,
            reason: format!(
                "Injection 模式 {}（继续转发，仅记录）",
                hit.pattern_id
            ),
        });
    }

    SecurityVerdict::Allow
}

/// 从 JSON body 递归提取所有字符串拼成单文本（用于安全检查）
pub fn collect_request_text(body: &Value) -> String {
    let mut buf = String::new();
    walk_value(body, &mut buf);
    buf
}

fn walk_value(v: &Value, buf: &mut String) {
    match v {
        Value::String(s) => {
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(s);
        }
        Value::Array(arr) => {
            for item in arr {
                walk_value(item, buf);
            }
        }
        Value::Object(map) => {
            for (_k, val) in map {
                walk_value(val, buf);
            }
        }
        _ => {}
    }
}

// ──────────────────────────────────────────────────────────────────
// 事件 / 日志持久化
// ──────────────────────────────────────────────────────────────────

pub fn record_intercept(
    app: &AppState,
    meta: &EventMeta,
    agent_id: Option<String>,
    prompt_snippet: Option<String>,
) {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &app.db {
            let ev = crate::storage::models::InterceptEvent {
                id: 0,
                timestamp: now_rfc3339(),
                event_type: meta.event_type.into(),
                severity: meta.severity.into(),
                signal_class: meta.signal_class.into(),
                rule_id: Some(meta.rule_id.clone()),
                mitre_attack_id: meta.mitre_attack_id.clone(),
                confidence: "high".into(),
                details: meta.reason.clone(),
                evidence: None,
                prompt_snippet,
                agent_id,
                session_id: None,
            };
            if let Err(e) = crate::storage::queries::intercept::insert(db, &ev) {
                tracing::warn!(error = %e, "intercept_events insert failed");
            }
            return;
        }
    }
    let _ = (app, meta, agent_id, prompt_snippet);
}

#[derive(Debug, Clone)]
pub struct RequestLogEntry {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub format: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub blocked: bool,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub latency_ms: i64,
}

pub fn record_request_log(app: &AppState, entry: RequestLogEntry) -> Option<i64> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &app.db {
            let row = crate::storage::models::RequestLog {
                id: 0,
                timestamp: now_rfc3339(),
                agent_id: entry.agent_id,
                format: entry.format,
                provider: entry.provider,
                model: entry.model,
                endpoint: entry.endpoint,
                method: entry.method,
                status_code: entry.status_code,
                blocked: entry.blocked,
                bytes_in: entry.bytes_in,
                bytes_out: entry.bytes_out,
                latency_ms: entry.latency_ms,
            };
            match crate::storage::queries::usage::insert_request_log(db, &row) {
                Ok(id) => return Some(id),
                Err(e) => {
                    tracing::warn!(error = %e, "request_logs insert failed");
                }
            }
            return None;
        }
    }
    let _ = (app, entry);
    None
}

#[derive(Debug, Clone)]
pub struct TokenUsageEntry {
    pub request_log_id: Option<i64>,
    pub agent_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read: u32,
    pub cache_creation: u32,
    pub cost_usd: f64,
}

pub fn record_token_usage(app: &AppState, entry: TokenUsageEntry) {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &app.db {
            if let Err(e) = crate::storage::queries::usage::insert_token_usage(
                db,
                entry.request_log_id,
                entry.agent_id.as_deref(),
                &entry.provider,
                &entry.model,
                entry.input_tokens,
                entry.output_tokens,
                entry.cache_read,
                entry.cache_creation,
                entry.cost_usd,
            ) {
                tracing::warn!(error = %e, "token_usage insert failed");
            }
            return;
        }
    }
    let _ = (app, entry);
}

// ──────────────────────────────────────────────────────────────────
// 价格表（W13.7）—— 简化估算；W14 接云端定价
// ──────────────────────────────────────────────────────────────────

/// 返回 (input_per_1m, output_per_1m) USD；未知模型返回 (0, 0)
pub fn lookup_price(provider: &str, model: &str) -> (f64, f64) {
    let p = provider.to_ascii_lowercase();
    let m = model.to_ascii_lowercase();
    // 主流模型粗粒度估算（2026 价表，仅供参考）
    match (p.as_str(), m.as_str()) {
        (_, "gpt-4o") => (5.0, 15.0),
        (_, "gpt-4o-mini") => (0.15, 0.60),
        (_, "gpt-4-turbo") => (10.0, 30.0),
        (_, "gpt-3.5-turbo") => (0.50, 1.50),
        (_, m) if m.contains("claude-3-5-sonnet") => (3.0, 15.0),
        (_, m) if m.contains("claude-3-5-haiku") => (0.80, 4.0),
        (_, m) if m.contains("claude-3-opus") => (15.0, 75.0),
        (_, m) if m.contains("claude-3-sonnet") => (3.0, 15.0),
        (_, m) if m.contains("claude-3-haiku") => (0.25, 1.25),
        (_, m) if m.contains("gemini-1.5-pro") => (1.25, 5.0),
        (_, m) if m.contains("gemini-1.5-flash") => (0.075, 0.30),
        _ => (0.0, 0.0),
    }
}

pub fn estimate_cost_usd(provider: &str, model: &str, input: u32, output: u32) -> f64 {
    let (in_price, out_price) = lookup_price(provider, model);
    (input as f64 / 1_000_000.0) * in_price + (output as f64 / 1_000_000.0) * out_price
}

// ──────────────────────────────────────────────────────────────────
// 时间格式化（避免引入 chrono 依赖）
// ──────────────────────────────────────────────────────────────────

pub fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (y, mo, d, h, mi, s) = unix_to_ymdhms(secs);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, mo, d, h, mi, s
    )
}

fn unix_to_ymdhms(t: i64) -> (i32, u32, u32, u32, u32, u32) {
    let secs = t.rem_euclid(60) as u32;
    let mins = (t / 60).rem_euclid(60) as u32;
    let hours = (t / 3600).rem_euclid(24) as u32;
    let mut days = (t / 86400) as i64;
    let mut year = 1970i32;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days >= dy {
            days -= dy;
            year += 1;
        } else if days < 0 {
            year -= 1;
            let dy_prev = if is_leap(year) { 366 } else { 365 };
            days += dy_prev;
        } else {
            break;
        }
    }
    let months_days = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    let mut day_idx = days;
    for (i, &md) in months_days.iter().enumerate() {
        if day_idx < md {
            month = (i as u32) + 1;
            break;
        }
        day_idx -= md;
    }
    let day = (day_idx + 1) as u32;
    (year, month, day, hours, mins, secs)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ──────────────────────────────────────────────────────────────────
// 路径识别
// ──────────────────────────────────────────────────────────────────

/// 给定 path，判断是否是已知 LLM 兼容路径（需要 buffer body 做检查）
pub fn is_llm_request_path(path: &str) -> bool {
    path.ends_with("/chat/completions")
        || path.ends_with("/messages")
        || path.ends_with("/responses")
        || path.ends_with(":generateContent")
        || path.ends_with(":streamGenerateContent")
        || path.ends_with("/api/chat")
        || path.ends_with("/api/generate")
}

pub fn protocol_from_path(path: &str) -> &'static str {
    if path.ends_with("/messages") {
        "anthropic"
    } else if path.ends_with("/responses") {
        "openai_responses"
    } else if path.contains(":generateContent") || path.contains("googleapis.com") {
        "gemini"
    } else if path.ends_with("/api/chat") || path.ends_with("/api/generate") {
        "ollama"
    } else {
        "openai"
    }
}
