//! Discovered agents queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use crate::storage::models::DiscoveredAgent;
use rusqlite::params;

pub fn upsert(conn: &SharedConnection, ag: &DiscoveredAgent) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO discovered_agents(
            platform, agent_name, config_path, process_name, last_seen,
            mcp_servers, config_hash, status
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(platform, agent_name) DO UPDATE SET
            config_path = excluded.config_path,
            process_name = excluded.process_name,
            last_seen = excluded.last_seen,
            mcp_servers = excluded.mcp_servers,
            config_hash = excluded.config_hash,
            status = excluded.status",
        params![
            ag.platform, ag.agent_name, ag.config_path, ag.process_name,
            ag.last_seen, ag.mcp_servers, ag.config_hash, ag.status,
        ],
    )?;
    Ok(())
}

pub fn list_all(conn: &SharedConnection) -> rusqlite::Result<Vec<DiscoveredAgent>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, platform, agent_name, config_path, process_name, last_seen,
                mcp_servers, config_hash, status
         FROM discovered_agents ORDER BY last_seen DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DiscoveredAgent {
            id: row.get(0)?,
            platform: row.get(1)?,
            agent_name: row.get(2)?,
            config_path: row.get(3)?,
            process_name: row.get(4)?,
            last_seen: row.get(5)?,
            mcp_servers: row.get(6)?,
            config_hash: row.get(7)?,
            status: row.get(8)?,
        })
    })?;
    rows.collect()
}
