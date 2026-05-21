//! IPC: Token 用量统计（CC-Switch 借鉴）
//!
//! 命令族：
//!   - get_usage_summary       — 今日总览（输入/输出/缓存/成本/请求/拦截）
//!   - get_usage_trends        — 时间序列（最近 N 天每天）
//!   - get_usage_by_provider   — 按 provider 汇总
//!   - get_usage_by_model      — 按 provider+model 汇总

use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Default)]
pub struct UsageSummary {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cost_usd: f64,
    pub request_count: u32,
    pub blocked_count: u32,
}

#[tauri::command]
pub fn get_usage_summary(_state: State<AppState>) -> AppResult<UsageSummary> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok((input, output, cache, cost, req, blk)) =
                crate::storage::queries::usage::today_summary(db)
            {
                return Ok(UsageSummary {
                    input_tokens: input,
                    output_tokens: output,
                    cache_read_tokens: cache,
                    cost_usd: cost,
                    request_count: req,
                    blocked_count: blk,
                });
            }
        }
    }
    Ok(UsageSummary::default())
}

#[derive(Serialize)]
pub struct UsageDay {
    pub date: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
}

#[tauri::command]
pub fn get_usage_trends(
    _state: State<AppState>,
    days: Option<u32>,
) -> AppResult<Vec<UsageDay>> {
    let days = days.unwrap_or(14);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::usage::daily_usage_summary(db, days) {
                // 反转使时间正序（旧 → 新），便于前端绘制趋势
                let mut v: Vec<UsageDay> = rows.into_iter().map(|(d, i, o, c)| UsageDay {
                    date: d,
                    input_tokens: i,
                    output_tokens: o,
                    cost_usd: c,
                }).collect();
                v.reverse();
                return Ok(v);
            }
        }
    }

    let _ = days;
    Ok(vec![])
}

#[derive(Serialize)]
pub struct UsageProviderRow {
    pub provider: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
    pub request_count: u32,
}

#[tauri::command]
pub fn get_usage_by_provider(
    _state: State<AppState>,
    days: Option<u32>,
) -> AppResult<Vec<UsageProviderRow>> {
    let days = days.unwrap_or(30);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::usage::usage_by_provider(db, days) {
                return Ok(rows.into_iter().map(|(p, i, o, c, r)| UsageProviderRow {
                    provider: p,
                    input_tokens: i,
                    output_tokens: o,
                    cost_usd: c,
                    request_count: r,
                }).collect());
            }
        }
    }

    let _ = days;
    Ok(vec![])
}

#[derive(Serialize)]
pub struct UsageModelRow {
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
    pub request_count: u32,
}

#[tauri::command]
pub fn get_usage_by_model(
    _state: State<AppState>,
    days: Option<u32>,
) -> AppResult<Vec<UsageModelRow>> {
    let days = days.unwrap_or(30);

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::usage::usage_by_model(db, days) {
                return Ok(rows.into_iter().map(|(p, m, i, o, c, r)| UsageModelRow {
                    provider: p,
                    model: m,
                    input_tokens: i,
                    output_tokens: o,
                    cost_usd: c,
                    request_count: r,
                }).collect());
            }
        }
    }

    let _ = days;
    Ok(vec![])
}
