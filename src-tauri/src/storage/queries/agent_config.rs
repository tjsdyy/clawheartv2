//! Agent config snapshots / batches queries (W7)
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRow {
    pub id: String,
    pub batch_id: String,
    pub agent_id: String,
    pub agent_platform: String,
    pub config_path: String,
    pub config_kind: String,
    pub before_value: String,
    pub after_value: String,
    pub applied_at: String,
    pub rolled_back_at: Option<String>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub batch_id: String,
    pub profile_id: Option<String>,
    pub agent_count: u32,
    pub applied_at: String,
    pub fully_rolled_back: bool,
}

fn row_from(row: &rusqlite::Row) -> rusqlite::Result<SnapshotRow> {
    Ok(SnapshotRow {
        id: row.get(0)?,
        batch_id: row.get(1)?,
        agent_id: row.get(2)?,
        agent_platform: row.get(3)?,
        config_path: row.get(4)?,
        config_kind: row.get(5)?,
        before_value: row.get(6)?,
        after_value: row.get(7)?,
        applied_at: row.get(8)?,
        rolled_back_at: row.get(9)?,
        profile_id: row.get(10)?,
    })
}

const SELECT_COLS: &str =
    "id, batch_id, agent_id, agent_platform, config_path, config_kind, \
     before_value, after_value, applied_at, rolled_back_at, profile_id";

#[allow(clippy::too_many_arguments)]
pub fn insert_snapshot(
    conn: &SharedConnection,
    id: &str,
    batch_id: &str,
    agent_id: &str,
    agent_platform: &str,
    config_path: &str,
    config_kind: &str,
    before_value: &str,
    after_value: &str,
    profile_id: Option<&str>,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO agent_config_snapshots
            (id, batch_id, agent_id, agent_platform, config_path, config_kind,
             before_value, after_value, applied_at, rolled_back_at, profile_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), NULL, ?9)",
        params![
            id, batch_id, agent_id, agent_platform, config_path,
            config_kind, before_value, after_value, profile_id,
        ],
    )?;
    Ok(())
}

pub fn mark_rolled_back(conn: &SharedConnection, snapshot_id: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE agent_config_snapshots SET rolled_back_at = datetime('now') WHERE id = ?1",
        params![snapshot_id],
    )?;
    Ok(())
}

pub fn list_snapshots_by_batch(
    conn: &SharedConnection,
    batch_id: &str,
) -> rusqlite::Result<Vec<SnapshotRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(&format!(
        "SELECT {} FROM agent_config_snapshots WHERE batch_id = ?1 ORDER BY applied_at",
        SELECT_COLS
    ))?;
    let rows = stmt.query_map(params![batch_id], row_from)?;
    rows.collect()
}

pub fn list_recent_snapshots(
    conn: &SharedConnection,
    limit: i64,
) -> rusqlite::Result<Vec<SnapshotRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(&format!(
        "SELECT {} FROM agent_config_snapshots ORDER BY applied_at DESC LIMIT ?1",
        SELECT_COLS
    ))?;
    let rows = stmt.query_map(params![limit], row_from)?;
    rows.collect()
}

pub fn get_snapshot(
    conn: &SharedConnection,
    id: &str,
) -> rusqlite::Result<Option<SnapshotRow>> {
    let c = conn.lock().unwrap();
    match c.query_row(
        &format!(
            "SELECT {} FROM agent_config_snapshots WHERE id = ?1",
            SELECT_COLS
        ),
        params![id],
        row_from,
    ) {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 列出所有 batch 摘要（按时间倒序，最多 50 条）
pub fn list_batches(conn: &SharedConnection) -> rusqlite::Result<Vec<BatchSummary>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT batch_id,
                MIN(profile_id),
                COUNT(*) as cnt,
                MIN(applied_at) as applied,
                SUM(CASE WHEN rolled_back_at IS NULL THEN 0 ELSE 1 END) as rolled
         FROM agent_config_snapshots
         GROUP BY batch_id
         ORDER BY applied DESC
         LIMIT 50",
    )?;
    let rows = stmt.query_map([], |row| {
        let cnt: u32 = row.get(2)?;
        let rolled: u32 = row.get(4)?;
        Ok(BatchSummary {
            batch_id: row.get(0)?,
            profile_id: row.get(1)?,
            agent_count: cnt,
            applied_at: row.get(3)?,
            fully_rolled_back: rolled == cnt,
        })
    })?;
    rows.collect()
}
