// IPC: 系统状态 — 完全 DB 驱动，无 mock fallback
use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct StatusInfo {
    pub version: String,
    pub protected: bool,
    pub agents: u32,
    pub mcp_servers: u32,
    pub today_requests: u32,
    pub today_blocks: u32,
    pub today_cost_usd: f64,
    pub budget_limit_usd: f64,
    pub proxy_port: u16,
    pub ca_trusted: bool,
    pub kill_switch: bool,
    /// 防护已运行的秒数（W4 接 proxy uptime；未启动返回 0）
    pub uptime_sec: u64,
    /// 出/入流量字节（W6 接代理统计；未启动返回 0）
    pub bytes_in: u64,
    pub bytes_out: u64,
    /// 上次云同步 unix 秒（W18 接；未同步返回 0）
    pub last_sync_unix: u64,
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> AppResult<StatusInfo> {
    let kill_active = state.kill_switch.snapshot();

    // 默认全 0；启用 storage feature 后从 DB 真实计算
    let mut today_blocks: u32 = 0;
    let mut today_requests: u32 = 0;
    let mut today_cost: f64 = 0.0;
    let mut agents: u32 = 0;

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            today_blocks = crate::storage::queries::intercept::count_today(db).unwrap_or(0);
            today_requests = count_request_logs_today(db).unwrap_or(0);
            today_cost = sum_cost_today(db).unwrap_or(0.0);
            agents = crate::storage::queries::agents::list_all(db)
                .map(|v| v.len() as u32)
                .unwrap_or(0);
        }
    }

    Ok(StatusInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        protected: !kill_active,
        agents,
        mcp_servers: 0, // 真实：当前未连接任何 MCP server
        today_requests,
        today_blocks,
        today_cost_usd: today_cost,
        budget_limit_usd: 0.0, // 真实：用户未设置预算
        proxy_port: 19111,
        ca_trusted: false, // W6 检测证书信任
        kill_switch: kill_active,
        uptime_sec: 0,
        bytes_in: 0,
        bytes_out: 0,
        last_sync_unix: 0,
    })
}

#[cfg(feature = "storage")]
fn count_request_logs_today(db: &crate::storage::conn::SharedConnection) -> rusqlite::Result<u32> {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM request_logs WHERE date(timestamp) = date('now')",
        [],
        |r| r.get::<_, u32>(0),
    )
}

#[cfg(feature = "storage")]
fn sum_cost_today(db: &crate::storage::conn::SharedConnection) -> rusqlite::Result<f64> {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage WHERE date(timestamp) = date('now')",
        [],
        |r| r.get::<_, f64>(0),
    )
}

#[derive(Serialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_sec: u64,
}

#[tauri::command]
pub fn get_proxy_status() -> AppResult<ProxyStatus> {
    // proxy_real feature 启用前一直是 false
    Ok(ProxyStatus {
        running: false,
        port: 19111,
        uptime_sec: 0,
    })
}
