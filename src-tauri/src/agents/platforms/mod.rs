//! 各 AI Agent 平台的发现实现

pub mod claude;
pub mod codex;
pub mod cursor;
pub mod detect;
pub mod gemini;
pub mod hermes;
pub mod openclaw;
pub mod openeva;
pub mod unknown;
pub mod windsurf;

use super::DiscoveredAgent;

pub trait PlatformScanner: Send + Sync {
    fn id(&self) -> &'static str;
    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String>;
}

/// 通用 helper：检查 $HOME 下某路径是否存在。
pub fn home_path(rel: &str) -> Option<std::path::PathBuf> {
    dirs_home().map(|h| h.join(rel))
}

pub(crate) fn dirs_home() -> Option<std::path::PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(std::path::PathBuf::from)
    }
}
