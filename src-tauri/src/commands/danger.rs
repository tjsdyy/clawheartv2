//! IPC: 危险指令
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct DangerRuleItem {
    pub rule_id: String,
    pub description: String,
    pub mitre_attack_id: Option<String>,
    pub enabled: bool,
}

#[tauri::command]
pub fn list_danger_commands(state: State<AppState>) -> AppResult<Vec<DangerRuleItem>> {
    // 优先从 DB 读（含用户 toggle 状态）；DB 为空时回退到内置 BUILTIN_RULES
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            if let Ok(rows) = crate::storage::queries::danger::list(db) {
                if !rows.is_empty() {
                    return Ok(rows
                        .into_iter()
                        .map(|r| DangerRuleItem {
                            rule_id: r.rule_id,
                            description: r.pattern,
                            mitre_attack_id: r.mitre_attack_id,
                            enabled: r.enabled,
                        })
                        .collect());
                }
            }
        }
    }
    let _ = state;
    // 回退：从内置规则常量返回（首次启动 / storage 未启用）
    Ok(crate::security::danger::BUILTIN_RULES
        .iter()
        .map(|r| DangerRuleItem {
            rule_id: r.id.into(),
            description: r.description.into(),
            mitre_attack_id: r.mitre_attack_id.map(String::from),
            enabled: true,
        })
        .collect())
}

#[tauri::command]
pub fn toggle_danger_command(
    state: State<AppState>,
    rule_id: String,
    enabled: bool,
) -> AppResult<()> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            crate::storage::queries::danger::toggle(db, &rule_id, enabled)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = (state, rule_id, enabled);
    Ok(())
}

#[tauri::command]
pub fn sync_danger_commands(state: State<AppState>) -> AppResult<u32> {
    // 同步内置规则到 DB（用于首次启动）；W17 接云端规则更新
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            crate::storage::queries::danger::seed_builtin(db)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(crate::security::danger::BUILTIN_RULES.len() as u32);
        }
    }
    let _ = state;
    Ok(0)
}
