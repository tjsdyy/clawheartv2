//! Cursor IDE 配置探测器
//!
//! 配置路径（按平台优先级）：
//!   macOS:   ~/Library/Application Support/Cursor/User/settings.json
//!   Linux:   ~/.config/Cursor/User/settings.json
//!   Windows: %APPDATA%\Cursor\User\settings.json
//!
//! 配置键：
//!   cursor.openaiBaseUrl  → ClawHeart 反向代理 URL
//!   cursor.openaiApiKey   → 虚拟 key
//!
//! 写入模式：直接 patch JSON 对象顶层键；保留其他配置不动。

use crate::agents::config_probe::*;
use crate::agents::DiscoveredAgent;
use std::path::PathBuf;

pub struct CursorProbe;

const KEY_BASE_URL: &str = "cursor.openaiBaseUrl";
const KEY_API_KEY: &str = "cursor.openaiApiKey";

fn settings_path() -> Option<PathBuf> {
    let home = home_dir()?;
    let candidates = [
        home.join("Library/Application Support/Cursor/User/settings.json"),
        home.join(".config/Cursor/User/settings.json"),
        home.join("AppData/Roaming/Cursor/User/settings.json"),
    ];
    for p in candidates {
        if p.exists() {
            return Some(p);
        }
    }
    // 没找到现成的：返回第一个（macOS 默认）用于"未发现也提示路径"
    Some(home.join("Library/Application Support/Cursor/User/settings.json"))
}

fn read_json(path: &std::path::Path) -> serde_json::Value {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| {
            serde_json::Value::Object(serde_json::Map::new())
        }),
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    }
}

impl ConfigProbe for CursorProbe {
    fn platform(&self) -> &'static str { "cursor" }

    fn inspect(&self, agent: &DiscoveredAgent) -> ProbeResult {
        let path = settings_path();
        let path_str = path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let exists = path.as_ref().map(|p| p.exists()).unwrap_or(false);

        let mut current_base_url = None;
        let mut current_key_present = false;
        if let Some(p) = &path {
            if exists {
                let json = read_json(p);
                if let Some(serde_json::Value::String(s)) = json.get(KEY_BASE_URL) {
                    current_base_url = Some(s.clone());
                }
                if let Some(serde_json::Value::String(s)) = json.get(KEY_API_KEY) {
                    if !s.is_empty() {
                        current_key_present = true;
                    }
                }
            }
        }

        let mut warnings = Vec::new();
        if !exists {
            warnings
                .push("Cursor 配置文件不存在（用户尚未启动过 Cursor 或路径异常）".into());
        }

        ProbeResult {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "cursor".into(),
            agent_name: agent.agent_name.clone(),
            current_base_url,
            current_key_present,
            config_source: ConfigSource::JsonFile {
                path: path_str,
                json_path: KEY_BASE_URL.into(),
            },
            writable: exists,
            probe_available: true,
            warnings,
        }
    }

    fn plan_overwrite(
        &self,
        agent: &DiscoveredAgent,
        target: &OverwriteTarget<'_>,
    ) -> Option<ConfigPatch> {
        let path = settings_path()?;
        let exists = path.exists();
        let before_json = if exists { read_json(&path) } else {
            serde_json::Value::Object(serde_json::Map::new())
        };
        let before = serde_json::to_string_pretty(&before_json)
            .unwrap_or_else(|_| "{}".into());

        let mut after_json = before_json.clone();
        let _ = json_set_path(
            &mut after_json,
            KEY_BASE_URL,
            serde_json::Value::String(target.base_url.to_string()),
        );
        let _ = json_set_path(
            &mut after_json,
            KEY_API_KEY,
            serde_json::Value::String(target.virtual_key.to_string()),
        );
        let after = serde_json::to_string_pretty(&after_json)
            .unwrap_or_else(|_| "{}".into());

        let diff_lines = make_diff_lines(&before, &after);
        let risk = if !exists {
            PatchRisk::Caution
        } else {
            PatchRisk::Safe
        };

        Some(ConfigPatch {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "cursor".into(),
            agent_name: agent.agent_name.clone(),
            source: ConfigSource::JsonFile {
                path: path.to_string_lossy().to_string(),
                json_path: KEY_BASE_URL.into(),
            },
            before,
            after,
            diff_lines,
            risk_level: risk,
        })
    }

    fn apply(&self, patch: &ConfigPatch, dry_run: bool) -> Result<AppliedPatch, String> {
        let real_path = match &patch.source {
            ConfigSource::JsonFile { path, .. } => PathBuf::from(path),
            _ => return Err("Cursor Probe 只支持 JsonFile 源".into()),
        };
        let write_path = if dry_run {
            dry_run_path("cursor", "User/settings.json")
        } else {
            real_path.clone()
        };
        ensure_parent(&write_path)?;
        std::fs::write(&write_path, &patch.after).map_err(|e| format!("写入失败：{}", e))?;
        tracing::info!(
            platform = "cursor",
            dry_run,
            path = %write_path.to_string_lossy(),
            "config patch applied"
        );
        Ok(AppliedPatch {
            config_path: write_path.to_string_lossy().to_string(),
            before_value: patch.before.clone(),
            after_value: patch.after.clone(),
            dry_run,
        })
    }

    fn rollback(
        &self,
        config_path: &str,
        before_value: &str,
        dry_run: bool,
    ) -> Result<(), String> {
        let path = if dry_run {
            dry_run_path("cursor", "User/settings.json")
        } else {
            PathBuf::from(config_path)
        };
        ensure_parent(&path)?;
        std::fs::write(&path, before_value).map_err(|e| format!("回滚失败：{}", e))?;
        Ok(())
    }

    fn inspect_with_credential(
        &self,
        _agent: &DiscoveredAgent,
    ) -> Option<CredentialReadResult> {
        let path = settings_path()?;
        if !path.exists() {
            return None;
        }
        let json = read_json(&path);
        let base_url = json.get(KEY_BASE_URL).and_then(|v| v.as_str()).map(String::from)?;
        let api_key = json.get(KEY_API_KEY).and_then(|v| v.as_str()).map(String::from)?;
        if api_key.is_empty() {
            return None;
        }
        Some(CredentialReadResult {
            base_url,
            api_key,
            source_path: path.to_string_lossy().to_string(),
            source_label: format!("Cursor · settings.json[{}]", KEY_BASE_URL),
        })
    }
}
