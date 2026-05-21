//! IPC: Agent ↔ Channel 分配
//!
//! - list_agent_channels(agent_id) → 返回该 Agent 已分配的 profile_id 列表
//! - list_channel_agents(profile_id) → 返回某渠道分配给了哪些 agent
//! - list_all_assignments() → 全部分配关系（渠道库视图用）
//! - assign_channel(agent_id, profile_id) → 单条分配
//! - unassign_channel(agent_id, profile_id) → 取消分配
//! - replace_agent_channels(agent_id, profile_ids) → 批量替换 Agent 的分配

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentDto {
    pub agent_id: String,
    pub profile_id: String,
}

#[tauri::command]
pub fn list_agent_channels(
    _state: State<AppState>,
    agent_id: String,
) -> AppResult<Vec<String>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            let ids = crate::storage::queries::assignments::list_by_agent(db, &agent_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(ids);
        }
    }
    Ok(vec![])
}

#[tauri::command]
pub fn list_channel_agents(
    _state: State<AppState>,
    profile_id: String,
) -> AppResult<Vec<String>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            let ids = crate::storage::queries::assignments::list_by_profile(db, &profile_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(ids);
        }
    }
    Ok(vec![])
}

#[tauri::command]
pub fn list_all_assignments(
    _state: State<AppState>,
) -> AppResult<Vec<AssignmentDto>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            let pairs = crate::storage::queries::assignments::list_all(db)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(pairs
                .into_iter()
                .map(|(a, p)| AssignmentDto { agent_id: a, profile_id: p })
                .collect());
        }
    }
    Ok(vec![])
}

#[tauri::command]
pub fn assign_channel(
    _state: State<AppState>,
    agent_id: String,
    profile_id: String,
) -> AppResult<bool> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            crate::storage::queries::assignments::assign(db, &agent_id, &profile_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(true);
        }
    }
    Err(AppError::NotImplemented("storage feature disabled"))
}

#[tauri::command]
pub fn unassign_channel(
    _state: State<AppState>,
    agent_id: String,
    profile_id: String,
) -> AppResult<bool> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            crate::storage::queries::assignments::unassign(db, &agent_id, &profile_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(true);
        }
    }
    Err(AppError::NotImplemented("storage feature disabled"))
}

#[tauri::command]
pub fn replace_agent_channels(
    _state: State<AppState>,
    agent_id: String,
    profile_ids: Vec<String>,
) -> AppResult<bool> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            crate::storage::queries::assignments::replace_for_agent(
                db, &agent_id, &profile_ids,
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(true);
        }
    }
    Err(AppError::NotImplemented("storage feature disabled"))
}
