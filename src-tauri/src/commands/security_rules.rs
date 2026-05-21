//! IPC: 安全规则配置菜单
//!
//! 把 5 类常量规则（danger / injection / credential / skill / audit）的
//! enabled / action 状态暴露给前端，存到 `security_rule_overrides` 表。
use crate::error::{AppError, AppResult};
use crate::security::rule_registry::{list_all_descriptors, RuleDescriptor};
use crate::state::AppState;
use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

#[derive(Serialize)]
pub struct SecurityRuleRow {
    /// 来自代码常量的不可变描述
    #[serde(flatten)]
    pub descriptor: RuleDescriptor,
    /// 用户覆盖：true 启用 / false 禁用（缺省 = true）
    pub enabled: bool,
    /// 用户覆盖动作：None = 用默认；Some("block"/"warn"/"skip")
    pub action_override: Option<String>,
    /// 过去 7 天命中次数（从 intercept_events 聚合）
    pub hits_7d: u32,
}

#[derive(Serialize)]
pub struct SecurityRuleCounts {
    pub kind: String,
    pub total: u32,
    pub enabled: u32,
    pub hits_7d: u32,
}

#[tauri::command]
pub fn list_security_rules(_state: State<AppState>) -> AppResult<Vec<SecurityRuleRow>> {
    let descriptors = list_all_descriptors();
    let mut overrides: HashMap<(String, String), (bool, Option<String>)> = HashMap::new();
    let mut hits: HashMap<String, u32> = HashMap::new();

    #[cfg(feature = "storage")]
    if let Some(db) = &_state.db {
        if let Ok(rows) = crate::storage::queries::security_rules::list_all(db) {
            for r in rows {
                overrides.insert((r.rule_kind, r.rule_id), (r.enabled, r.action));
            }
        }
        // 聚合最近 7 天命中
        if let Ok(c) = db.lock() {
            if let Ok(mut stmt) = c.prepare(
                "SELECT rule_id, COUNT(*) FROM intercept_events
                 WHERE rule_id IS NOT NULL
                   AND timestamp >= datetime('now', '-7 days')
                 GROUP BY rule_id",
            ) {
                if let Ok(iter) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                }) {
                    let collected: Vec<(String, i64)> = iter.flatten().collect();
                    for (id, n) in collected {
                        hits.insert(id, n.max(0) as u32);
                    }
                }
            }
        }
    }

    Ok(build_rows(descriptors, overrides, hits))
}

fn build_rows(
    descriptors: Vec<RuleDescriptor>,
    overrides: HashMap<(String, String), (bool, Option<String>)>,
    hits: HashMap<String, u32>,
) -> Vec<SecurityRuleRow> {
    descriptors
        .into_iter()
        .map(|d| {
            let key = (d.kind.as_str().to_string(), d.id.clone());
            let (enabled, action_override) = overrides.get(&key).cloned().unwrap_or((true, None));
            let hits_7d = *hits.get(&d.id).unwrap_or(&0);
            SecurityRuleRow {
                descriptor: d,
                enabled,
                action_override,
                hits_7d,
            }
        })
        .collect()
}

/// 启用 / 禁用一条规则
#[tauri::command]
pub fn toggle_security_rule(
    _state: State<AppState>,
    rule_kind: String,
    rule_id: String,
    enabled: bool,
) -> AppResult<()> {
    if !valid_kind(&rule_kind) {
        return Err(AppError::Other(format!("无效的 rule_kind：{}", rule_kind)));
    }
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            // 读现有 action_override（toggle 不动作 action）
            let action = crate::storage::queries::security_rules::list_by_kind(db, &rule_kind)
                .ok()
                .and_then(|rows| rows.into_iter().find(|r| r.rule_id == rule_id))
                .and_then(|r| r.action);
            crate::storage::queries::security_rules::upsert(
                db,
                &rule_kind,
                &rule_id,
                enabled,
                action.as_deref(),
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = (rule_kind, rule_id, enabled);
    Ok(())
}

/// 设置触发动作覆盖（block / warn / skip / null = 恢复默认）
#[tauri::command]
pub fn set_rule_action(
    _state: State<AppState>,
    rule_kind: String,
    rule_id: String,
    action: Option<String>,
) -> AppResult<()> {
    if !valid_kind(&rule_kind) {
        return Err(AppError::Other(format!("无效的 rule_kind：{}", rule_kind)));
    }
    if let Some(a) = action.as_deref() {
        if !matches!(a, "block" | "warn" | "skip") {
            return Err(AppError::Other(format!(
                "无效的 action：{}（仅允许 block / warn / skip）",
                a
            )));
        }
    }
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            // 读现有 enabled
            let enabled = crate::storage::queries::security_rules::list_by_kind(db, &rule_kind)
                .ok()
                .and_then(|rows| rows.into_iter().find(|r| r.rule_id == rule_id))
                .map(|r| r.enabled)
                .unwrap_or(true);
            crate::storage::queries::security_rules::upsert(
                db,
                &rule_kind,
                &rule_id,
                enabled,
                action.as_deref(),
            )
            .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = (rule_kind, rule_id, action);
    Ok(())
}

/// 恢复单条规则默认（删除 override）
#[tauri::command]
pub fn reset_rule(
    _state: State<AppState>,
    rule_kind: String,
    rule_id: String,
) -> AppResult<()> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            crate::storage::queries::security_rules::reset(db, &rule_kind, &rule_id)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = (rule_kind, rule_id);
    Ok(())
}

/// 恢复整组默认（清空某 kind 的全部 override）
#[tauri::command]
pub fn reset_rule_kind(_state: State<AppState>, rule_kind: String) -> AppResult<()> {
    if !valid_kind(&rule_kind) {
        return Err(AppError::Other(format!("无效的 rule_kind：{}", rule_kind)));
    }
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            crate::storage::queries::security_rules::reset_kind(db, &rule_kind)
                .map_err(|e| AppError::Other(format!("DB error: {}", e)))?;
            return Ok(());
        }
    }
    let _ = rule_kind;
    Ok(())
}

fn valid_kind(kind: &str) -> bool {
    matches!(
        kind,
        "danger" | "injection" | "credential" | "skill" | "audit"
    )
}
