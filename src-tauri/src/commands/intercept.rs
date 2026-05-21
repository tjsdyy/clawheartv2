//! IPC: 拦截事件 / 请求日志 / 导出
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct InterceptEventListItem {
    pub id: u64,
    pub timestamp: String,
    pub severity: String,
    pub event_type: String,
    pub label: String,
    pub agent_id: Option<String>,
    pub rule_id: Option<String>,
    pub mitre_attack_id: Option<String>,
}

#[tauri::command]
pub fn list_intercept_events(
    _state: State<AppState>,
    limit: Option<u32>,
    _offset: Option<u32>,
) -> AppResult<Vec<InterceptEventListItem>> {
    let limit = limit.unwrap_or(50);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::intercept::list_recent(db, limit) {
                return Ok(rows.into_iter().map(|r| InterceptEventListItem {
                    id: r.id as u64,
                    timestamp: r.timestamp,
                    severity: r.severity,
                    event_type: r.event_type,
                    label: r.details.chars().take(120).collect(),
                    agent_id: r.agent_id,
                    rule_id: r.rule_id,
                    mitre_attack_id: r.mitre_attack_id,
                }).collect());
            }
        }
    }

    let _ = limit;
    Ok(vec![])
}

#[tauri::command]
pub fn get_intercept_event(
    _state: State<AppState>,
    id: u64,
) -> AppResult<serde_json::Value> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(Some(ev)) = crate::storage::queries::intercept::get_by_id(db, id as i64) {
                return Ok(serde_json::json!({
                    "id": ev.id,
                    "timestamp": ev.timestamp,
                    "event_type": ev.event_type,
                    "severity": ev.severity,
                    "signal_class": ev.signal_class,
                    "rule_id": ev.rule_id,
                    "mitre_attack_id": ev.mitre_attack_id,
                    "confidence": ev.confidence,
                    "details": ev.details,
                    "evidence": ev.evidence,
                    "prompt_snippet": ev.prompt_snippet,
                    "agent_id": ev.agent_id,
                    "session_id": ev.session_id,
                }));
            }
            return Ok(serde_json::json!({ "id": id, "error": "not_found" }));
        }
    }
    let _ = id;
    Ok(serde_json::json!({}))
}

#[derive(Serialize)]
pub struct RequestLogListItem {
    pub id: i64,
    pub timestamp: String,
    pub agent_id: Option<String>,
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

#[tauri::command]
pub fn list_request_logs(
    _state: State<AppState>,
    limit: Option<u32>,
) -> AppResult<Vec<RequestLogListItem>> {
    let limit = limit.unwrap_or(30);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::usage::list_request_logs(db, limit) {
                return Ok(rows.into_iter().map(|r| RequestLogListItem {
                    id: r.id, timestamp: r.timestamp, agent_id: r.agent_id,
                    format: r.format, provider: r.provider, model: r.model,
                    endpoint: r.endpoint, method: r.method, status_code: r.status_code,
                    blocked: r.blocked, bytes_in: r.bytes_in, bytes_out: r.bytes_out,
                    latency_ms: r.latency_ms,
                }).collect());
            }
        }
    }

    let _ = limit;
    Ok(vec![])
}

/// 真实导出 — 把今日 intercept_events + request_logs 写到 ~/.clawheart-v2/exports/。
/// 支持 JSON（完整），CSV/STIX/SARIF 在 W6 实现。
#[tauri::command]
pub fn export_request_logs(_state: State<AppState>, format: String) -> AppResult<String> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            let dir = crate::state::data_dir().join("exports");
            std::fs::create_dir_all(&dir).map_err(AppError::Io)?;

            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let path = dir.join(format!("clawheart-export-{}.{}", ts, file_ext(&format)));

            match format.as_str() {
                "json" => write_json(db, &path)?,
                "csv" => write_csv(db, &path)?,
                other => {
                    return Err(AppError::NotImplemented(match other {
                        "stix" => "STIX 2.1 导出在 W6 实现",
                        "sarif" => "SARIF 导出在 W6 实现",
                        _ => "未知格式",
                    }));
                }
            }
            return Ok(path.to_string_lossy().into_owned());
        }
    }
    let _ = format;
    Err(AppError::NotImplemented("storage feature 未启用"))
}

fn file_ext(format: &str) -> &str {
    match format {
        "json" => "json",
        "csv" => "csv",
        "stix" => "stix.json",
        "sarif" => "sarif.json",
        _ => "txt",
    }
}

#[cfg(feature = "storage")]
fn write_json(db: &crate::storage::conn::SharedConnection, path: &std::path::Path) -> AppResult<()> {
    let events = crate::storage::queries::intercept::list_recent(db, 1000).unwrap_or_default();
    let logs = crate::storage::queries::usage::list_request_logs(db, 1000).unwrap_or_default();

    let payload = serde_json::json!({
        "exported_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "tool": "ClawHeart Desktop",
        "version": env!("CARGO_PKG_VERSION"),
        "redaction_policy": "class-preserving placeholders <pl:CLASS:N>",
        "intercept_events": events,
        "request_logs": logs,
    });
    let s = serde_json::to_string_pretty(&payload).map_err(AppError::Serde)?;
    std::fs::write(path, s).map_err(AppError::Io)?;
    Ok(())
}

#[cfg(feature = "storage")]
fn write_csv(db: &crate::storage::conn::SharedConnection, path: &std::path::Path) -> AppResult<()> {
    let logs = crate::storage::queries::usage::list_request_logs(db, 5000).unwrap_or_default();
    let mut buf = String::from("id,timestamp,agent,format,provider,model,endpoint,status,blocked,bytes_in,bytes_out,latency_ms\n");
    for r in &logs {
        buf.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.id, csv_escape(&r.timestamp), csv_escape(r.agent_id.as_deref().unwrap_or("")),
            csv_escape(&r.format),
            csv_escape(r.provider.as_deref().unwrap_or("")),
            csv_escape(r.model.as_deref().unwrap_or("")),
            csv_escape(&r.endpoint),
            r.status_code, r.blocked,
            r.bytes_in, r.bytes_out, r.latency_ms,
        ));
    }
    std::fs::write(path, buf).map_err(AppError::Io)?;
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.into()
    }
}
