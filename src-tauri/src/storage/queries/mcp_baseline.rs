//! MCP tool baseline persistence
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;

pub fn freeze(conn: &SharedConnection, session: &str, server: &str, tool: &str, hash: &str, capability: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO mcp_tool_baselines(session_id, server_id, tool_name, description_hash, capability, frozen_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(session_id, server_id, tool_name) DO NOTHING",
        params![session, server, tool, hash, capability],
    )?;
    Ok(())
}

pub fn baseline_hash(conn: &SharedConnection, session: &str, server: &str, tool: &str) -> rusqlite::Result<Option<String>> {
    let c = conn.lock().unwrap();
    match c.query_row(
        "SELECT description_hash FROM mcp_tool_baselines
         WHERE session_id = ?1 AND server_id = ?2 AND tool_name = ?3",
        params![session, server, tool],
        |row| row.get::<_, String>(0),
    ) {
        Ok(h) => Ok(Some(h)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}
