//! IPC: 安全公告
use crate::error::AppResult;
use serde::Serialize;

#[derive(Serialize)]
pub struct AdvisoryListItem {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub cvss_score: Option<f64>,
    pub published: String,
    pub matched_locally: bool,
    pub dismissed: bool,
}

#[tauri::command]
pub fn list_advisories() -> AppResult<Vec<AdvisoryListItem>> { Ok(vec![]) }

#[tauri::command]
pub fn acknowledge_advisory(_id: String) -> AppResult<()> { Ok(()) }

#[tauri::command]
pub fn subscribe_feed(_url: String) -> AppResult<()> { Ok(()) }
