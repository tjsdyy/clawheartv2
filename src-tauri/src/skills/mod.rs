//! 本机技能发现 / 安全扫描 / 备份打包。
//!
//! - [`discover`] 通配扫描 `~/.<agent>/skills/` 目录，自动捕获任何 Agent
//! - [`manifest`] 解析 SKILL.md frontmatter
//! - [`backup`] 把选中的技能打包为 zip

pub mod discover;
pub mod manifest;
pub mod backup;
pub mod manage;

pub use discover::{discover_all, DiscoveredSkill};
pub use backup::{backup_skills, BackupResult};
pub use manage::{
    ensure_ssot_dir, move_to_ssot, repair_binding, ssot_config, toggle_binding,
    uninstall_skill, SsotConfig,
};
