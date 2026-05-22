//! Claude Code 配置探测器
//!
//! 配置路径：
//!   ~/.claude/settings.json
//!
//! 配置键：
//!   env.ANTHROPIC_BASE_URL  → ClawHeart 反向代理 URL
//!   env.ANTHROPIC_API_KEY   → 虚拟 key
//!
//! Claude Code 使用 settings.json 的 env 段注入环境变量（参考 v1 配置）

use crate::agents::config_probe::*;
use crate::agents::probes::ChannelCandidate;
use crate::agents::DiscoveredAgent;
use std::path::PathBuf;

pub struct ClaudeCodeProbe;

const PATH_BASE_URL: &str = "env.ANTHROPIC_BASE_URL";
const PATH_API_KEY: &str = "env.ANTHROPIC_API_KEY";

/// 从 ~/.claude/settings.json 提取唯一的 anthropic 渠道（单 provider）
pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(path) = home_dir().map(|h| h.join(".claude/settings.json")) else {
        return vec![];
    };
    let Ok(content) = std::fs::read_to_string(&path) else { return vec![]; };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { return vec![]; };

    let env = v.get("env").and_then(|e| e.as_object());
    let base_url = env
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let Some(base_url) = base_url else { return vec![]; };

    let api_key = env
        .and_then(|e| {
            e.get("ANTHROPIC_AUTH_TOKEN")
                .or_else(|| e.get("ANTHROPIC_API_KEY"))
        })
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let mut warnings = Vec::new();
    if api_key.is_none() {
        warnings.push("env 中未找到 ANTHROPIC_AUTH_TOKEN，导入后需手动配置".into());
    }

    vec![ChannelCandidate {
        id: "claude:anthropic".to_string(),
        name: "Anthropic".to_string(),
        source_agent_id: agent_id.to_string(),
        source_platform: "claude".to_string(),
        base_url,
        api_key,
        protocol: "anthropic".to_string(),
        default_model: None,
        provider_kind: "anthropic".to_string(),
        already_exists: false,
        warnings,
    }]
}

fn settings_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".claude/settings.json"))
}

fn read_json(path: &std::path::Path) -> serde_json::Value {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| {
            serde_json::Value::Object(serde_json::Map::new())
        }),
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    }
}

impl ConfigProbe for ClaudeCodeProbe {
    fn platform(&self) -> &'static str { "claude" }

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
                if let Some(serde_json::Value::String(s)) = json_get_path(&json, PATH_BASE_URL) {
                    current_base_url = Some(s);
                }
                if let Some(serde_json::Value::String(s)) = json_get_path(&json, PATH_API_KEY) {
                    if !s.is_empty() {
                        current_key_present = true;
                    }
                }
            }
        }

        let mut warnings = Vec::new();
        if !exists {
            warnings.push("Claude Code settings.json 不存在".into());
        }

        ProbeResult {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "claude".into(),
            agent_name: agent.agent_name.clone(),
            current_base_url,
            current_key_present,
            config_source: ConfigSource::JsonFile {
                path: path_str,
                json_path: PATH_BASE_URL.into(),
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
            PATH_BASE_URL,
            serde_json::Value::String(target.base_url.to_string()),
        );
        let _ = json_set_path(
            &mut after_json,
            PATH_API_KEY,
            serde_json::Value::String(target.virtual_key.to_string()),
        );
        let after = serde_json::to_string_pretty(&after_json)
            .unwrap_or_else(|_| "{}".into());

        let diff_lines = make_diff_lines(&before, &after);
        let risk = if !exists { PatchRisk::Caution } else { PatchRisk::Safe };

        Some(ConfigPatch {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "claude".into(),
            agent_name: agent.agent_name.clone(),
            source: ConfigSource::JsonFile {
                path: path.to_string_lossy().to_string(),
                json_path: PATH_BASE_URL.into(),
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
            _ => return Err("Claude Code Probe 只支持 JsonFile 源".into()),
        };
        let write_path = if dry_run {
            dry_run_path("claude", "settings.json")
        } else {
            real_path.clone()
        };
        ensure_parent(&write_path)?;
        std::fs::write(&write_path, &patch.after).map_err(|e| format!("写入失败：{}", e))?;
        tracing::info!(
            platform = "claude",
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
            dry_run_path("claude", "settings.json")
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
        let base_url = match json_get_path(&json, PATH_BASE_URL) {
            Some(serde_json::Value::String(s)) => s,
            _ => return None,
        };
        let api_key = match json_get_path(&json, PATH_API_KEY) {
            Some(serde_json::Value::String(s)) if !s.is_empty() => s,
            _ => return None,
        };
        Some(CredentialReadResult {
            base_url,
            api_key,
            source_path: path.to_string_lossy().to_string(),
            source_label: format!("Claude Code · settings.json[{}]", PATH_BASE_URL),
        })
    }
}
