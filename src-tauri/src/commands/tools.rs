// IPC: 工具矩阵徽章 + 拦截事件
use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct ToolDescriptor {
    pub id: String,
    pub status: String, // active | coming_soon
    pub badge_kind: Option<String>,
    pub badge_value: Option<String>,
}

/// 返回工具矩阵每张卡片的真实徽章数据。
/// 没数据时返回 `None`（前端显示无徽章，不再硬编码"2 新""142"等假数字）。
#[tauri::command]
pub fn list_tools(_state: State<AppState>) -> AppResult<Vec<ToolDescriptor>> {
    let mut monitor_badge: (Option<String>, Option<String>) = (None, None);
    let mut scan_badge: (Option<String>, Option<String>) = (None, None);
    let mut skills_badge: (Option<String>, Option<String>) = (None, None);
    let mut advisory_badge: (Option<String>, Option<String>) = (None, None);
    let mut logs_badge: (Option<String>, Option<String>) = (None, None);
    let mut budget_badge: (Option<String>, Option<String>) = (None, None);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            // 监控：今日 critical 拦截数 → "N 新"
            let crit = count_today_critical(db).unwrap_or(0);
            if crit > 0 {
                monitor_badge = (Some("alert".into()), Some(format!("{} 新", crit)));
            }

            // 扫描：最近一次扫描的 failed 数 → "N ●"
            if let Some(last) = last_scan_failed(db) {
                if last > 0 {
                    scan_badge = (Some("count".into()), Some(format!("{} ●", last)));
                }
            }

            // 技能：已启用数 / 总数
            let (enabled, total) = count_skills(db).unwrap_or((0, 0));
            if total > 0 {
                skills_badge = (Some("count".into()), Some(format!("{}/{}", enabled, total)));
            }

            // 公告：未忽略的 high+ 数 → "N 高"
            let pending_adv = count_pending_advisories(db).unwrap_or(0);
            if pending_adv > 0 {
                advisory_badge = (Some("alert-high".into()), Some(format!("{}", pending_adv)));
            }

            // 日志：今日请求数
            let today_req = count_request_logs_today(db).unwrap_or(0);
            if today_req > 0 {
                logs_badge = (
                    Some("count".into()),
                    Some(format_count(today_req)),
                );
            }

            // 预算：今日全局用量 / 上限百分比
            if let Some(pct) = primary_budget_pct(db) {
                budget_badge = (Some("count".into()), Some(format!("{}%", pct)));
            }
        }
    }

    Ok(vec![
        ToolDescriptor { id: "monitor".into(),  status: "active".into(), badge_kind: monitor_badge.0,  badge_value: monitor_badge.1 },
        ToolDescriptor { id: "scan".into(),     status: "active".into(), badge_kind: scan_badge.0,     badge_value: scan_badge.1 },
        ToolDescriptor { id: "skills".into(),   status: "active".into(), badge_kind: skills_badge.0,   badge_value: skills_badge.1 },
        ToolDescriptor { id: "advisory".into(), status: "active".into(), badge_kind: advisory_badge.0, badge_value: advisory_badge.1 },
        ToolDescriptor { id: "logs".into(),     status: "active".into(), badge_kind: logs_badge.0,     badge_value: logs_badge.1 },
        ToolDescriptor { id: "budget".into(),   status: "active".into(), badge_kind: budget_badge.0,   badge_value: budget_badge.1 },
        ToolDescriptor { id: "audit".into(),    status: "active".into(), badge_kind: None, badge_value: None },
        ToolDescriptor { id: "token_verify".into(), status: "coming_soon".into(), badge_kind: Some("soon".into()), badge_value: Some("SOON".into()) },
        ToolDescriptor { id: "openclaw".into(), status: "active".into(), badge_kind: None, badge_value: None },
        ToolDescriptor { id: "relay".into(),    status: "coming_soon".into(), badge_kind: Some("soon".into()), badge_value: Some("SOON".into()) },
        ToolDescriptor { id: "policy".into(),   status: "coming_soon".into(), badge_kind: Some("soon".into()), badge_value: Some("V2.2".into()) },
    ])
}

#[cfg(feature = "storage")]
fn count_today_critical(db: &crate::storage::conn::SharedConnection) -> rusqlite::Result<u32> {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM intercept_events
         WHERE date(timestamp) = date('now') AND severity = 'critical'",
        [], |r| r.get::<_, u32>(0),
    )
}

#[cfg(feature = "storage")]
fn last_scan_failed(db: &crate::storage::conn::SharedConnection) -> Option<i64> {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT failed FROM scan_runs ORDER BY started_at DESC LIMIT 1",
        [], |r| r.get::<_, i64>(0),
    ).ok()
}

#[cfg(feature = "storage")]
fn count_skills(db: &crate::storage::conn::SharedConnection) -> rusqlite::Result<(u32, u32)> {
    let c = db.lock().unwrap();
    let total: u32 = c.query_row("SELECT COUNT(*) FROM skills", [], |r| r.get(0))?;
    let enabled: u32 = c.query_row(
        "SELECT COUNT(*) FROM skills WHERE user_enabled = 1",
        [], |r| r.get(0),
    )?;
    Ok((enabled, total))
}

#[cfg(feature = "storage")]
fn count_pending_advisories(db: &crate::storage::conn::SharedConnection) -> rusqlite::Result<u32> {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM security_advisories
         WHERE dismissed = 0 AND severity IN ('critical', 'high')",
        [], |r| r.get::<_, u32>(0),
    )
}

#[cfg(feature = "storage")]
fn count_request_logs_today(db: &crate::storage::conn::SharedConnection) -> rusqlite::Result<u32> {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM request_logs WHERE date(timestamp) = date('now')",
        [], |r| r.get::<_, u32>(0),
    )
}

#[cfg(feature = "storage")]
fn primary_budget_pct(db: &crate::storage::conn::SharedConnection) -> Option<u32> {
    let c = db.lock().unwrap();
    let (limit, _provider): (f64, String) = c.query_row(
        "SELECT limit_usd, provider FROM budget_rules
         WHERE enabled = 1 AND period = 'daily' AND provider = 'global'
         ORDER BY id LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?)),
    ).ok()?;
    let used: f64 = c.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage WHERE date(timestamp) = date('now')",
        [], |r| r.get(0),
    ).ok()?;
    if limit <= 0.0 { return None; }
    Some(((used / limit) * 100.0).round() as u32)
}

fn format_count(n: u32) -> String {
    if n >= 1000 {
        format!("{:.1}K", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

#[derive(Serialize)]
pub struct InterceptEvent {
    pub id: u32,
    pub timestamp: String,
    pub severity: String,
    pub event_type: String,
    pub label: String,
    pub agent: String,
}

/// 返回最近 10 条拦截事件。表空 = 空数组（不再 mock）。
#[tauri::command]
pub fn list_recent_events(_state: State<AppState>) -> AppResult<Vec<InterceptEvent>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::intercept::list_recent(db, 10) {
                return Ok(rows.into_iter().map(|r| InterceptEvent {
                    id: r.id as u32,
                    timestamp: r.timestamp,
                    severity: r.severity,
                    event_type: r.event_type,
                    label: r.details.chars().take(120).collect(),
                    agent: r.agent_id.unwrap_or_default(),
                }).collect());
            }
        }
    }
    Ok(vec![])
}

#[tauri::command]
pub fn trigger_kill_switch(state: State<AppState>, activate: bool) -> AppResult<bool> {
    if activate {
        state.kill_switch.activate_api();
        tracing::warn!("Kill switch ACTIVATED via UI");
    } else {
        state.kill_switch.reset();
        tracing::info!("Kill switch reset via UI");
    }
    Ok(state.kill_switch.snapshot())
}
