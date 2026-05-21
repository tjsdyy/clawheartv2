//! IPC: 预算 / 用量
use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct BudgetRuleItem {
    pub id: u32,
    pub provider: String,
    pub model: Option<String>,
    pub period: String,
    pub limit_usd: f64,
    pub used_usd: f64,
    pub enabled: bool,
}

#[tauri::command]
pub fn list_budget_rules(_state: State<AppState>) -> AppResult<Vec<BudgetRuleItem>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::budget::list(db) {
                return Ok(rows.into_iter().map(|r| {
                    let used = crate::storage::queries::budget::used_today(
                        db, &r.provider, r.model.as_deref(),
                    ).unwrap_or(0.0);
                    BudgetRuleItem {
                        id: r.id as u32, provider: r.provider, model: r.model,
                        period: r.period, limit_usd: r.limit_usd, used_usd: used,
                        enabled: r.enabled,
                    }
                }).collect());
            }
        }
    }
    // 真实：无 DB 或表空 → 空数组（不再返回 mock）
    Ok(vec![])
}

#[tauri::command]
pub fn set_budget_rule(_state: State<AppState>, rule: serde_json::Value) -> AppResult<()> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            let row = crate::storage::queries::budget::BudgetRow {
                id: 0,
                provider: rule.get("provider").and_then(|v| v.as_str()).unwrap_or("global").into(),
                model: rule.get("model").and_then(|v| v.as_str()).map(String::from),
                period: rule.get("period").and_then(|v| v.as_str()).unwrap_or("daily").into(),
                limit_usd: rule.get("limit_usd").and_then(|v| v.as_f64()).unwrap_or(0.0),
                enabled: rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            };
            crate::storage::queries::budget::insert(db, &row)
                .map_err(|e| crate::error::AppError::Other(e.to_string()))?;
            return Ok(());
        }
    }
    let _ = rule;
    Ok(())
}

#[derive(Serialize)]
pub struct TokenUsageDay {
    pub date: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
}

#[tauri::command]
pub fn get_token_usage(
    _state: State<AppState>,
    days: Option<u32>,
) -> AppResult<Vec<TokenUsageDay>> {
    let n = days.unwrap_or(7).clamp(1, 90);
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::usage::daily_usage_summary(db, n) {
                return Ok(rows
                    .into_iter()
                    .map(|(date, input, output, cost)| TokenUsageDay {
                        date,
                        input_tokens: input,
                        output_tokens: output,
                        cost_usd: cost,
                    })
                    .collect());
            }
        }
    }
    let _ = n;
    Ok(vec![])
}
