//! Intercept events queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use crate::storage::models::InterceptEvent;
use rusqlite::params;

pub fn insert(conn: &SharedConnection, ev: &InterceptEvent) -> rusqlite::Result<i64> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO intercept_events(
            timestamp, event_type, severity, signal_class, rule_id, mitre_attack_id,
            confidence, details, evidence, prompt_snippet, agent_id, session_id, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))",
        params![
            ev.timestamp, ev.event_type, ev.severity, ev.signal_class,
            ev.rule_id, ev.mitre_attack_id, ev.confidence, ev.details,
            ev.evidence, ev.prompt_snippet, ev.agent_id, ev.session_id,
        ],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn list_recent(conn: &SharedConnection, limit: u32) -> rusqlite::Result<Vec<InterceptEvent>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, timestamp, event_type, severity, signal_class, rule_id, mitre_attack_id,
                confidence, details, evidence, prompt_snippet, agent_id, session_id
         FROM intercept_events
         ORDER BY timestamp DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(InterceptEvent {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            event_type: row.get(2)?,
            severity: row.get(3)?,
            signal_class: row.get(4)?,
            rule_id: row.get(5)?,
            mitre_attack_id: row.get(6)?,
            confidence: row.get(7)?,
            details: row.get(8)?,
            evidence: row.get(9)?,
            prompt_snippet: row.get(10)?,
            agent_id: row.get(11)?,
            session_id: row.get(12)?,
        })
    })?;
    rows.collect()
}

pub fn get_by_id(conn: &SharedConnection, id: i64) -> rusqlite::Result<Option<InterceptEvent>> {
    let c = conn.lock().unwrap();
    match c.query_row(
        "SELECT id, timestamp, event_type, severity, signal_class, rule_id, mitre_attack_id,
                confidence, details, evidence, prompt_snippet, agent_id, session_id
         FROM intercept_events WHERE id = ?1",
        params![id],
        |row| {
            Ok(InterceptEvent {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                severity: row.get(3)?,
                signal_class: row.get(4)?,
                rule_id: row.get(5)?,
                mitre_attack_id: row.get(6)?,
                confidence: row.get(7)?,
                details: row.get(8)?,
                evidence: row.get(9)?,
                prompt_snippet: row.get(10)?,
                agent_id: row.get(11)?,
                session_id: row.get(12)?,
            })
        },
    ) {
        Ok(e) => Ok(Some(e)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn count_today(conn: &SharedConnection) -> rusqlite::Result<u32> {
    let c = conn.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM intercept_events WHERE date(timestamp) = date('now')",
        [],
        |row| row.get::<_, u32>(0),
    )
}
