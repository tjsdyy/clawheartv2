//! 启动时种子内置数据
//!
//! 触发：DB 首次打开时（schema_migrations 中没有 seed_version 记录则跑）。
//! 内容：
//!   - BUILTIN_RULES（30 危险指令）写入 danger_commands
//!   - 默认预算规则：global $10/day（用户可改）
//!   - 默认 settings：theme / language / proxy_port

#![cfg(feature = "storage")]

use crate::storage::conn::SharedConnection;
use rusqlite::params;

const SEED_VERSION: i32 = 1;

pub fn run_if_needed(conn: &SharedConnection) -> rusqlite::Result<bool> {
    let already = {
        let c = conn.lock().unwrap();
        c.query_row(
            "SELECT 1 FROM settings WHERE key = ?1",
            params![format!("seed_v{}", SEED_VERSION)],
            |_| Ok(true),
        ).unwrap_or(false)
    };
    if already {
        return Ok(false);
    }

    seed_danger_rules(conn)?;
    seed_default_budget(conn)?;
    seed_default_settings(conn)?;

    crate::storage::queries::settings::set(
        conn,
        &format!("seed_v{}", SEED_VERSION),
        &chrono_now(),
    )?;
    Ok(true)
}

fn seed_danger_rules(conn: &SharedConnection) -> rusqlite::Result<()> {
    crate::storage::queries::danger::seed_builtin(conn)
}

fn seed_default_budget(conn: &SharedConnection) -> rusqlite::Result<()> {
    let c = conn.lock().unwrap();
    // 全局 $10/day（用户可在预算页改）
    c.execute(
        "INSERT OR IGNORE INTO budget_rules(provider, model, period, limit_usd, enabled)
         VALUES ('global', NULL, 'daily', 10.0, 1),
                ('global', NULL, 'monthly', 200.0, 1)",
        [],
    )?;
    Ok(())
}

fn seed_default_settings(conn: &SharedConnection) -> rusqlite::Result<()> {
    crate::storage::queries::settings::set(conn, "theme", "paper")?;
    crate::storage::queries::settings::set(conn, "language", "zh")?;
    crate::storage::queries::settings::set(conn, "proxy_port", "19111")?;
    Ok(())
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let t = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", t)
}
