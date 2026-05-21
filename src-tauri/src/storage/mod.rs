//! 数据层 — rusqlite + schema 迁移 + Keychain
//!
//! 启用 `storage` feature 后接入真实数据库（rusqlite bundled）+ OS Keychain（keyring crate）。

pub mod keychain;
pub mod migrations;
pub mod models;
pub mod queries;
pub mod seed;

#[cfg(feature = "storage")]
pub mod conn {
    use rusqlite::Connection;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    pub type SharedConnection = Arc<Mutex<Connection>>;

    /// 打开/创建 v2 数据库；启用 WAL + FK；幂等地跑 schema。
    pub fn open(path: &Path) -> rusqlite::Result<SharedConnection> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
        crate::storage::migrations::ensure_v2(&conn)?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    /// 默认路径：~/.clawheart-v2/clawheart.db
    pub fn default_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".clawheart-v2")
            .join("clawheart.db")
    }
}
