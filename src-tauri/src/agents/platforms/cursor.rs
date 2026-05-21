//! Cursor IDE 扫描器（跨平台 fallback 路径）
//!
//! 判定证据（按平台目录，进一步查 User/ 子目录的 settings/storage）：
//! - macOS: ~/Library/Application Support/Cursor/User/{settings.json,globalStorage/storage.json}
//! - Linux: ~/.config/Cursor/User/...
//! - Windows: %APPDATA%/Cursor/User/...

use super::detect::{detect_config, status_label, ConfigCandidate, ConfigFormat, DetectionStatus};
use super::{home_path, PlatformScanner};
use crate::agents::DiscoveredAgent;

pub struct CursorPlatform;

impl PlatformScanner for CursorPlatform {
    fn id(&self) -> &'static str { "cursor" }

    fn scan(&self) -> Result<Vec<DiscoveredAgent>, String> {
        let roots = [
            home_path("Library/Application Support/Cursor"),
            home_path(".config/Cursor"),
            home_path("AppData/Roaming/Cursor"),
        ];

        let mut candidates: Vec<ConfigCandidate> = Vec::new();
        for root in roots.into_iter().flatten() {
            candidates.push(ConfigCandidate::new(
                root.join("User/settings.json"),
                ConfigFormat::JsonRelaxed,
            ));
            candidates.push(ConfigCandidate::new(
                root.join("User/globalStorage/storage.json"),
                ConfigFormat::JsonRelaxed,
            ));
            candidates.push(ConfigCandidate::new(
                root.join("User/keybindings.json"),
                ConfigFormat::JsonRelaxed,
            ));
        }

        let res = detect_config(&candidates);
        if res.status == DetectionStatus::NotFound {
            return Ok(vec![]);
        }

        let path = res
            .matched_path
            .as_ref()
            .and_then(|p| p.parent().and_then(|p| p.parent()))
            .map(|p| p.to_path_buf())
            .or_else(|| res.matched_path.clone())
            .unwrap_or_default();

        Ok(vec![DiscoveredAgent {
            platform: "cursor".into(),
            agent_name: "Cursor".into(),
            config_path: Some(path.to_string_lossy().to_string()),
            process_name: Some("Cursor".into()),
            last_seen: now_unix(),
            mcp_servers: vec![],
            config_hash: None,
            status: status_label(&res.status).into(),
            discovery_signals: vec![],
        }])
    }
}

fn now_unix() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{}", now)
}
