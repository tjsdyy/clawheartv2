//! `clawheart status` — 整机状态摘要
use serde::Serialize;
use std::path::PathBuf;

use super::output::{CliResult, Output};

#[derive(Serialize)]
struct StatusDto {
    version: String,
    desktop_running: bool,
    db_path: String,
    db_exists: bool,
    db_size_bytes: u64,
    discovered_agents: usize,
    discovered_skills: usize,
}

pub fn execute(json: bool, db: Option<PathBuf>) -> CliResult {
    let db_path = db.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".clawheart-v2")
            .join("clawheart.db")
    });
    let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let db_exists = db_path.exists();

    let agents = clawheart_lib::agents::scanner::Scanner::with_default_platforms().scan_once();
    let skills = clawheart_lib::skills::discover::discover_all();

    let dto = StatusDto {
        version: env!("CARGO_PKG_VERSION").into(),
        desktop_running: detect_desktop_running(),
        db_path: db_path.to_string_lossy().into_owned(),
        db_exists,
        db_size_bytes: db_size,
        discovered_agents: agents.len(),
        discovered_skills: skills.len(),
    };

    let text = format!(
        "ClawHeart {}\n\n\
         Desktop GUI:       {}\n\
         DB:                {} ({})\n\
         发现的 Agent：     {} 个\n\
         本机技能：         {} 个",
        dto.version,
        if dto.desktop_running { "✓ 正在运行" } else { "未运行" },
        dto.db_path,
        if dto.db_exists {
            format!("{} KB", dto.db_size_bytes / 1024)
        } else {
            "不存在".into()
        },
        dto.discovered_agents,
        dto.discovered_skills,
    );

    Output::ok_with_text(dto, text).emit(json);
    Ok(())
}

#[cfg(feature = "agents_real")]
fn detect_desktop_running() -> bool {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.processes()
        .values()
        .any(|p| p.name().to_string_lossy().to_lowercase().contains("clawheart"))
}

#[cfg(not(feature = "agents_real"))]
fn detect_desktop_running() -> bool {
    false
}
