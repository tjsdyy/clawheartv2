//! v2 schema + v1→v2 迁移
//!
//! 一次性迁移逻辑：
//! 1. 备份 v1 到 ~/.clawheart/backups/clawheart-v1.db.bak
//! 2. 打开 v2 空库，跑全部 v2 schema
//! 3. 复制：danger_commands / conversation_history（不动）
//! 4. 拆分：local_settings → Keychain + settings
//! 5. 合并：disabled_skills + deprecated_skills + user_skills → skills
//! 6. 重命名：local_llm_mappings → llm_mappings
//! 7. 重塑：llm_usage_cost_events → request_logs（保留原始 JSON 在 raw 字段）
//! 8. 写 schema_migrations(version=2)

pub const V2_SCHEMA_SQL: &str = include_str!("../../schema/v2.sql");

#[cfg(feature = "storage")]
pub fn ensure_v2(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(V2_SCHEMA_SQL)?;
    Ok(())
}

#[cfg(feature = "storage")]
pub fn migrate_v1_to_v2(_v1_path: &std::path::Path, _v2_path: &std::path::Path) -> Result<MigrationReport, String> {
    // W3 实现
    Err("not implemented (W3)".into())
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct MigrationReport {
    pub copied_rows: u64,
    pub skipped_rows: u64,
    pub warnings: Vec<String>,
}
