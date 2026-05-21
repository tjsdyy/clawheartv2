//! IPC: Kill Switch — 4 源 OR
use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct KillSwitchStatus {
    pub active: bool,
}

#[tauri::command]
pub fn kill_switch_activate(state: State<AppState>) -> AppResult<KillSwitchStatus> {
    state.kill_switch.activate_api();
    tracing::warn!("Kill switch activated via IPC");
    Ok(KillSwitchStatus { active: true })
}

#[tauri::command]
pub fn kill_switch_reset(state: State<AppState>) -> AppResult<KillSwitchStatus> {
    state.kill_switch.reset();
    Ok(KillSwitchStatus { active: state.kill_switch.snapshot() })
}

#[tauri::command]
pub fn kill_switch_status(state: State<AppState>) -> AppResult<KillSwitchStatus> {
    Ok(KillSwitchStatus { active: state.kill_switch.snapshot() })
}
