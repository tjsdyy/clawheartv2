//! IPC: 安全扫描 — 真跑 65 个 check + 落库到 scan_runs
//!
//! 在跑每项 check 时 emit `scan:check_done` Tauri event 给前端做实时进度反馈。
use crate::error::AppResult;
use crate::security::scanner::{count_by_category, run_scan_with, CheckResult};
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

/// `scan:check_done` 事件负载
#[derive(Serialize, Clone)]
struct ScanProgress<'a> {
    index: u32,
    total: u32,
    result: &'a CheckResult,
}

/// `scan:run_started` 事件负载
#[derive(Serialize, Clone)]
struct ScanStarted {
    total: u32,
}

#[derive(Serialize)]
pub struct ScanItemGroup {
    pub category: String,
    pub label: String,
    pub count: usize,
}

#[tauri::command]
pub fn get_scan_items() -> AppResult<Vec<ScanItemGroup>> {
    Ok(count_by_category().iter().map(|(cat, n)| ScanItemGroup {
        category: format!("{:?}", cat),
        label: cat.label().into(),
        count: *n,
    }).collect())
}

#[derive(Serialize)]
pub struct ScanRunResult {
    pub run_id: u64,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub warned: u32,
    pub skipped: u32,
    pub results: Vec<CheckResult>,
}

/// 启动扫描 — async 命令，scanner 跑在 blocking pool 里
///
/// 设计要点：
/// - 同步 `fn` 命令会阻塞 IPC 主流程，emit 出来的事件直到命令返回前都到不了 webview
/// - 改 `async fn` 后 Tauri 在 async runtime 上跑，emit 立即可达
/// - `run_scan_with` 内部仍是同步 + CPU/IO 重，所以用 `spawn_blocking` 隔离
#[tauri::command]
pub async fn start_scan_run(
    state: State<'_, AppState>,
    app: AppHandle,
    items: Vec<String>,
) -> AppResult<ScanRunResult> {
    // 1) 在 blocking pool 跑 scanner；callback 内 emit 事件
    let app_for_cb = app.clone();
    let items_for_scan = items.clone();
    let run = tauri::async_runtime::spawn_blocking(move || {
        let ids: Vec<&str> = items_for_scan.iter().map(|s| s.as_str()).collect();
        run_scan_with(&ids, |index, total, result| {
            if index == 1 {
                let _ = app_for_cb.emit("scan:run_started", ScanStarted { total });
            }
            let _ = app_for_cb.emit(
                "scan:check_done",
                ScanProgress {
                    index,
                    total,
                    result,
                },
            );
            // 给 webview 主线程让出，让累积的事件能批刷给前端
            // 仅 100µs 量级，对总扫描时长几乎无影响
            std::thread::sleep(std::time::Duration::from_micros(200));
        })
    })
    .await
    .map_err(|e| crate::error::AppError::Other(format!("scan task error: {}", e)))?;

    // 2) DB 落库（回到主流程；可以再次访问 state）
    let mut run_id: u64 = 1;
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let items_json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if let Ok(id) = crate::storage::queries::scan::start(db, &items_json) {
                let results_json =
                    serde_json::to_string(&run.results).unwrap_or_else(|_| "[]".into());
                let _ = crate::storage::queries::scan::complete(
                    db,
                    id,
                    &results_json,
                    run.total_checks,
                    run.passed,
                    run.failed,
                    run.warned,
                    run.skipped,
                );
                run_id = id as u64;
            }
        }
    }
    #[cfg(not(feature = "storage"))]
    let _ = state;

    let _ = app.emit("scan:run_done", run_id);

    Ok(ScanRunResult {
        run_id,
        total: run.total_checks,
        passed: run.passed,
        failed: run.failed,
        warned: run.warned,
        skipped: run.skipped,
        results: run.results,
    })
}

#[derive(Serialize)]
pub struct ScanHistoryItem {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total: i64,
    pub passed: i64,
    pub failed: i64,
    pub warned: i64,
    pub skipped: i64,
}

#[tauri::command]
pub fn list_scan_history(_state: State<AppState>) -> AppResult<Vec<ScanHistoryItem>> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            if let Ok(rows) = crate::storage::queries::scan::list_recent(db, 20) {
                return Ok(rows.into_iter().map(|r| ScanHistoryItem {
                    id: r.id, started_at: r.started_at, completed_at: r.completed_at,
                    total: r.total_checks, passed: r.passed,
                    failed: r.failed, warned: r.warned, skipped: r.skipped,
                }).collect());
            }
        }
    }
    Ok(vec![])
}

#[tauri::command]
pub fn get_scan_progress(_run_id: u64) -> AppResult<f32> { Ok(1.0) }

#[derive(Serialize)]
pub struct ScanRunDetail {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub items: Vec<String>,
    /// 原样直传 results_json 字符串，前端自行 JSON.parse；
    /// 避免 CheckResult 含 &'static str 不能 Deserialize 的麻烦。
    pub results_json: String,
    pub total: i64,
    pub passed: i64,
    pub failed: i64,
    pub warned: i64,
    pub skipped: i64,
}

#[tauri::command]
pub fn get_scan_run(_state: State<AppState>, _id: i64) -> AppResult<ScanRunDetail> {
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &_state.db {
            let row = crate::storage::queries::scan::get_run(db, _id)
                .map_err(|e| crate::error::AppError::Other(format!("未找到扫描记录 #{}: {}", _id, e)))?;
            let items: Vec<String> = serde_json::from_str(&row.items_json).unwrap_or_default();
            return Ok(ScanRunDetail {
                id: row.id,
                started_at: row.started_at,
                completed_at: row.completed_at,
                items,
                results_json: row.results_json,
                total: row.total_checks,
                passed: row.passed,
                failed: row.failed,
                warned: row.warned,
                skipped: row.skipped,
            });
        }
    }
    Err(crate::error::AppError::Other("storage feature 未启用，无法查看历史详情".into()))
}
