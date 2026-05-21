//! Agent ↔ Channel 分配 N:M 关联表
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;

/// 列出某 Agent 已分配的所有 profile_id（按 assigned_at desc）
pub fn list_by_agent(
    conn: &SharedConnection,
    agent_id: &str,
) -> rusqlite::Result<Vec<String>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT profile_id FROM agent_channel_assignments
         WHERE agent_id = ?1 ORDER BY assigned_at DESC",
    )?;
    let rows = stmt.query_map(params![agent_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

/// 列出某 profile 已分配给的所有 agent_id
pub fn list_by_profile(
    conn: &SharedConnection,
    profile_id: &str,
) -> rusqlite::Result<Vec<String>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT agent_id FROM agent_channel_assignments
         WHERE profile_id = ?1 ORDER BY assigned_at DESC",
    )?;
    let rows = stmt.query_map(params![profile_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

/// 列出全部分配关系（用于渠道库视图汇总）
pub fn list_all(
    conn: &SharedConnection,
) -> rusqlite::Result<Vec<(String, String)>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT agent_id, profile_id FROM agent_channel_assignments",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}

/// 分配渠道给 Agent（已存在则更新 assigned_at）
pub fn assign(
    conn: &SharedConnection,
    agent_id: &str,
    profile_id: &str,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO agent_channel_assignments(agent_id, profile_id, assigned_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(agent_id, profile_id) DO UPDATE SET assigned_at = datetime('now')",
        params![agent_id, profile_id],
    )?;
    Ok(())
}

/// 取消某 Agent 的某条渠道分配
pub fn unassign(
    conn: &SharedConnection,
    agent_id: &str,
    profile_id: &str,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "DELETE FROM agent_channel_assignments
         WHERE agent_id = ?1 AND profile_id = ?2",
        params![agent_id, profile_id],
    )?;
    Ok(())
}

/// 批量替换某 Agent 的分配列表（用于"管理分配" UI）
pub fn replace_for_agent(
    conn: &SharedConnection,
    agent_id: &str,
    profile_ids: &[String],
) -> rusqlite::Result<()> {
    let mut c = conn.lock().unwrap();
    let tx = c.transaction()?;
    tx.execute(
        "DELETE FROM agent_channel_assignments WHERE agent_id = ?1",
        params![agent_id],
    )?;
    for pid in profile_ids {
        tx.execute(
            "INSERT INTO agent_channel_assignments(agent_id, profile_id, assigned_at)
             VALUES (?1, ?2, datetime('now'))",
            params![agent_id, pid],
        )?;
    }
    tx.commit()
}
