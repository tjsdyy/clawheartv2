//! 文件漂移监控 — ClawSec soul-guardian 借鉴
//!
//! 关键文件首次发现 → 记录 sha256 baseline（drift_baselines 表）。
//! fsnotify 触发后比对 hash → 不一致 = 漂移事件。
//! 用户可一键还原 baseline，或确认接受 → 更新 baseline。

use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct DriftRecord {
    pub path: String,
    pub baseline_sha256: String,
    pub current_sha256: String,
    pub detected_at: String,
}

/// 计算文件 sha256（hex 字符串，64 char）。
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(crate::security::sha256::hex_string_bytes(&bytes))
}

/// 关键文件清单（相对 $HOME，需在 W17 由 agents/scanner 拼接绝对路径）。
pub const CRITICAL_FILES: &[&str] = &[
    ".claude/CLAUDE.md",
    ".claude/MEMORY.md",
    ".claude/settings.json",
    ".claude/.mcp.json",
    ".codex/AGENTS.md",
    ".codex/config.json",
    ".cursor/mcp.json",
    ".gemini/GEMINI.md",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sha256_file_works() {
        let path = std::env::temp_dir().join("clawheart-drift-test.txt");
        std::fs::File::create(&path).unwrap().write_all(b"abc").unwrap();
        let hash = sha256_file(&path).unwrap();
        assert_eq!(hash, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let _ = std::fs::remove_file(&path);
    }
}
