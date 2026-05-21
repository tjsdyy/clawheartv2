//! Danger commands queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerRow {
    pub id: i64,
    pub rule_id: String,
    pub pattern: String,
    pub pattern_type: String,
    pub mitre_attack_id: Option<String>,
    pub enabled: bool,
    pub source: String,
}

pub fn list(conn: &SharedConnection) -> rusqlite::Result<Vec<DangerRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, rule_id, pattern, pattern_type, mitre_attack_id, enabled, source
         FROM danger_commands ORDER BY rule_id"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DangerRow {
            id: row.get(0)?,
            rule_id: row.get(1)?,
            pattern: row.get(2)?,
            pattern_type: row.get(3)?,
            mitre_attack_id: row.get(4)?,
            enabled: row.get::<_, i32>(5)? != 0,
            source: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn seed_builtin(conn: &SharedConnection) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    for rule in crate::security::danger::BUILTIN_RULES {
        c.execute(
            "INSERT OR IGNORE INTO danger_commands(rule_id, pattern, pattern_type, mitre_attack_id, enabled, source, created_at, updated_at)
             VALUES (?1, ?2, 'regex', ?3, 1, 'builtin', datetime('now'), datetime('now'))",
            params![rule.id, rule.pattern_raw, rule.mitre_attack_id],
        )?;
    }
    Ok(())
}

pub fn toggle(conn: &SharedConnection, rule_id: &str, enabled: bool) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE danger_commands SET enabled = ?1, updated_at = datetime('now') WHERE rule_id = ?2",
        params![enabled as i32, rule_id],
    )?;
    Ok(())
}
