//! IPC: 候选 Agent 决策持久化
//!
//! 当 Scanner 发现 ~/.<name>/ 含 AI 线索的未知平台目录时，会以 status=candidate 返回。
//! 用户在 UI 上点 "纳入管理" → confirm；点 "忽略" → ignore。决策存 settings 表中
//! key `agent.unknown_decisions`，值为 JSON 字典 `{ platform: "confirmed" | "ignored" }`。
//!
//! list_agents / discover_agents_now 读取此字典，做后置过滤：
//! - ignored → 从结果中剔除
//! - confirmed → status 从 candidate 升为 active（让 UI 当作常规 Agent 处理）

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

pub const DECISIONS_KEY: &str = "agent.unknown_decisions";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentDecision {
    Confirmed,
    Ignored,
}

impl AgentDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Ignored => "ignored",
        }
    }
}

pub fn load_decisions(_state: &AppState) -> HashMap<String, String> {
    #[cfg(feature = "storage")]
    {
        if let Some(conn) = &_state.db {
            if let Ok(Some(s)) = crate::storage::queries::settings::get(conn, DECISIONS_KEY) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&s) {
                    return map;
                }
            }
        }
    }
    HashMap::new()
}

fn save_decisions(
    _state: &AppState,
    decisions: &HashMap<String, String>,
) -> Result<(), AppError> {
    #[cfg(feature = "storage")]
    {
        if let Some(conn) = &_state.db {
            let s = serde_json::to_string(decisions)?;
            crate::storage::queries::settings::set(conn, DECISIONS_KEY, &s)
                .map_err(|e| AppError::Other(format!("settings.set failed: {}", e)))?;
            return Ok(());
        }
    }
    let _ = decisions;
    Err(AppError::Other("storage feature 未启用，决策无法持久化".into()))
}

#[tauri::command]
pub fn confirm_unknown_agent(
    state: State<AppState>,
    platform: String,
) -> AppResult<()> {
    let mut d = load_decisions(&state);
    d.insert(platform, AgentDecision::Confirmed.as_str().into());
    save_decisions(&state, &d)?;
    Ok(())
}

#[tauri::command]
pub fn ignore_unknown_agent(
    state: State<AppState>,
    platform: String,
) -> AppResult<()> {
    let mut d = load_decisions(&state);
    d.insert(platform, AgentDecision::Ignored.as_str().into());
    save_decisions(&state, &d)?;
    Ok(())
}

#[tauri::command]
pub fn reset_unknown_agent_decision(
    state: State<AppState>,
    platform: String,
) -> AppResult<()> {
    let mut d = load_decisions(&state);
    d.remove(&platform);
    save_decisions(&state, &d)?;
    Ok(())
}

#[tauri::command]
pub fn list_unknown_agent_decisions(
    state: State<AppState>,
) -> AppResult<HashMap<String, String>> {
    Ok(load_decisions(&state))
}

/// 应用决策过滤：扫描得到的 raw agents 列表 → 过滤后的列表
/// - ignored 的 candidate 被剔除
/// - confirmed 的 candidate status 升为 "active"
pub fn apply_decisions(
    mut agents: Vec<crate::agents::DiscoveredAgent>,
    decisions: &HashMap<String, String>,
) -> Vec<crate::agents::DiscoveredAgent> {
    agents.retain(|a| {
        if a.status != "candidate" {
            return true;
        }
        decisions.get(&a.platform).map(|s| s.as_str()) != Some("ignored")
    });
    for a in &mut agents {
        if a.status == "candidate"
            && decisions.get(&a.platform).map(|s| s.as_str()) == Some("confirmed")
        {
            a.status = "active".into();
        }
    }
    agents
}
