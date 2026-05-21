//! Scan run history queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScanRunRow {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total_checks: i64,
    pub passed: i64,
    pub failed: i64,
    pub warned: i64,
    pub skipped: i64,
}

pub fn start(conn: &SharedConnection, items_json: &str) -> rusqlite::Result<i64> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO scan_runs(started_at, items_json) VALUES (datetime('now'), ?1)",
        params![items_json],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn complete(
    conn: &SharedConnection,
    id: i64,
    results_json: &str,
    total: u32,
    passed: u32,
    failed: u32,
    warned: u32,
    skipped: u32,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE scan_runs SET completed_at = datetime('now'), results_json = ?1,
                              total_checks = ?2, passed = ?3, failed = ?4, warned = ?5, skipped = ?6
         WHERE id = ?7",
        params![results_json, total, passed, failed, warned, skipped, id],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanRunDetail {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub items_json: String,
    pub results_json: String,
    pub total_checks: i64,
    pub passed: i64,
    pub failed: i64,
    pub warned: i64,
    pub skipped: i64,
}

pub fn get_run(conn: &SharedConnection, id: i64) -> rusqlite::Result<ScanRunDetail> {
    let c = conn.lock().unwrap();
    c.query_row(
        "SELECT id, started_at, completed_at, items_json, results_json,
                total_checks, passed, failed, warned, skipped
         FROM scan_runs WHERE id = ?1",
        params![id],
        |row| {
            Ok(ScanRunDetail {
                id: row.get(0)?,
                started_at: row.get(1)?,
                completed_at: row.get(2)?,
                items_json: row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "[]".into()),
                results_json: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "[]".into()),
                total_checks: row.get(5)?,
                passed: row.get(6)?,
                failed: row.get(7)?,
                warned: row.get(8)?,
                skipped: row.get(9)?,
            })
        },
    )
}

pub fn list_recent(conn: &SharedConnection, limit: u32) -> rusqlite::Result<Vec<ScanRunRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, started_at, completed_at, total_checks, passed, failed, warned, skipped
         FROM scan_runs ORDER BY started_at DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(ScanRunRow {
            id: row.get(0)?,
            started_at: row.get(1)?,
            completed_at: row.get(2)?,
            total_checks: row.get(3)?,
            passed: row.get(4)?,
            failed: row.get(5)?,
            warned: row.get(6)?,
            skipped: row.get(7)?,
        })
    })?;
    rows.collect()
}
