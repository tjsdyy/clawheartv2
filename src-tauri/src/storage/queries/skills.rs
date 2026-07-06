//! Skills queries
#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use crate::storage::models::Skill;
use rusqlite::{params, OptionalExtension};

pub fn list(conn: &SharedConnection) -> rusqlite::Result<Vec<Skill>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, slug, name, description, version, system_status, user_enabled,
                safety_label, scan_score, install_path, metadata, installed_at, updated_at
         FROM skills ORDER BY name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Skill {
            id: row.get(0)?,
            slug: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            version: row.get(4)?,
            system_status: row.get(5)?,
            user_enabled: row.get::<_, i32>(6)? != 0,
            safety_label: row.get(7)?,
            scan_score: row.get(8)?,
            install_path: row.get(9)?,
            metadata: row.get(10)?,
            installed_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.collect()
}

pub fn get(conn: &SharedConnection, slug: &str) -> rusqlite::Result<Option<Skill>> {
    let c = conn.lock().unwrap();
    c.query_row(
        "SELECT id, slug, name, description, version, system_status, user_enabled,
                safety_label, scan_score, install_path, metadata, installed_at, updated_at
         FROM skills WHERE slug = ?1",
        params![slug],
        |row| {
            Ok(Skill {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                version: row.get(4)?,
                system_status: row.get(5)?,
                user_enabled: row.get::<_, i32>(6)? != 0,
                safety_label: row.get(7)?,
                scan_score: row.get(8)?,
                install_path: row.get(9)?,
                metadata: row.get(10)?,
                installed_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )
    .optional()
}

pub fn upsert(conn: &SharedConnection, sk: &Skill) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO skills(slug, name, description, version, system_status, user_enabled,
                            safety_label, scan_score, install_path, metadata, installed_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, COALESCE(NULLIF(?11, ''), datetime('now')), datetime('now'))
         ON CONFLICT(slug) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            version = excluded.version,
            system_status = excluded.system_status,
            user_enabled = excluded.user_enabled,
            safety_label = excluded.safety_label,
            scan_score = excluded.scan_score,
            install_path = excluded.install_path,
            metadata = excluded.metadata,
            updated_at = datetime('now')",
        params![
            sk.slug, sk.name, sk.description, sk.version,
            sk.system_status, sk.user_enabled as i32,
            sk.safety_label, sk.scan_score, sk.install_path, sk.metadata, sk.installed_at,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &SharedConnection, slug: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute("DELETE FROM skill_scan_results WHERE skill_slug = ?1", params![slug])?;
    c.execute("DELETE FROM skills WHERE slug = ?1", params![slug])?;
    Ok(())
}

pub fn toggle(conn: &SharedConnection, slug: &str, enabled: bool) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE skills SET user_enabled = ?1, updated_at = datetime('now') WHERE slug = ?2",
        params![enabled as i32, slug],
    )?;
    Ok(())
}

pub fn set_safety(conn: &SharedConnection, slug: &str, label: &str) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE skills SET safety_label = ?1, updated_at = datetime('now') WHERE slug = ?2",
        params![label, slug],
    )?;
    Ok(())
}

pub fn insert_scan_result(
    conn: &SharedConnection,
    slug: &str,
    score: i32,
    risk_level: &str,
    blocked: bool,
    hard_triggers: &str,
    findings: &str,
) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO skill_scan_results(skill_slug, score, risk_level, blocked, hard_triggers, findings, scanned_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
        params![slug, score, risk_level, blocked as i32, hard_triggers, findings],
    )?;
    c.execute(
        "UPDATE skills
         SET scan_score = ?1,
             safety_label = ?2,
             updated_at = datetime('now')
         WHERE slug = ?3",
        params![score, if blocked { "disabled" } else if score >= 85 { "safe" } else { "warn" }, slug],
    )?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────
// Skill backups (备份历史)
// ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillBackupRow {
    pub id: i64,
    pub created_at: String,
    pub zip_path: String,
    pub skill_count: i64,
    pub total_bytes: i64,
    pub skill_ids: String,    // raw JSON array
    pub skill_names: String,  // raw JSON array
    pub zip_exists: bool,
}

pub fn insert_backup(
    conn: &SharedConnection,
    zip_path: &str,
    skill_count: u32,
    total_bytes: u64,
    skill_ids_json: &str,
    skill_names_json: &str,
) -> rusqlite::Result<i64> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO skill_backups(created_at, zip_path, skill_count, total_bytes, skill_ids, skill_names, zip_exists)
         VALUES (datetime('now'), ?1, ?2, ?3, ?4, ?5, 1)",
        params![
            zip_path,
            skill_count as i64,
            total_bytes as i64,
            skill_ids_json,
            skill_names_json,
        ],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn list_backups(conn: &SharedConnection, limit: u32) -> rusqlite::Result<Vec<SkillBackupRow>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id, created_at, zip_path, skill_count, total_bytes, skill_ids, skill_names, zip_exists
         FROM skill_backups ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(SkillBackupRow {
            id: row.get(0)?,
            created_at: row.get(1)?,
            zip_path: row.get(2)?,
            skill_count: row.get(3)?,
            total_bytes: row.get(4)?,
            skill_ids: row.get(5)?,
            skill_names: row.get(6)?,
            zip_exists: row.get::<_, i32>(7)? != 0,
        })
    })?;
    rows.collect()
}

pub fn delete_backup(conn: &SharedConnection, id: i64) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute("DELETE FROM skill_backups WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn mark_zip_missing(conn: &SharedConnection, id: i64) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "UPDATE skill_backups SET zip_exists = 0 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
