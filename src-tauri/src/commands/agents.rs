//! IPC: Agent 发现
use crate::agents::{scanner::Scanner, DiscoveredAgent};
use crate::commands::agent_decisions::{apply_decisions, load_decisions};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_agents(state: State<AppState>) -> AppResult<Vec<DiscoveredAgent>> {
    let scanner = Scanner::with_default_platforms();
    let raw = scanner.scan_once();
    let decisions = load_decisions(&state);
    Ok(apply_decisions(raw, &decisions))
}

#[tauri::command]
pub fn discover_agents_now(state: State<AppState>) -> AppResult<Vec<DiscoveredAgent>> {
    let scanner = Scanner::with_default_platforms();
    let raw = scanner.scan_once();
    let decisions = load_decisions(&state);
    Ok(apply_decisions(raw, &decisions))
}

/// 列出 Agent 上的 MCP servers
#[tauri::command]
pub fn list_mcp_servers(
    _state: State<AppState>,
    agent_id: Option<String>,
) -> AppResult<Vec<serde_json::Value>> {
    let agents = Scanner::with_default_platforms().scan_once();
    let mut out = Vec::new();
    for ag in agents {
        let id = format!("{}/{}", ag.platform, ag.agent_name);
        if let Some(f) = &agent_id {
            if f != &id {
                continue;
            }
        }
        for srv in &ag.mcp_servers {
            out.push(serde_json::json!({
                "agent_id": id,
                "agent_platform": ag.platform,
                "agent_name": ag.agent_name,
                "server_name": srv,
                "config_path": ag.config_path,
            }));
        }
    }
    let _ = _state;
    Ok(out)
}
