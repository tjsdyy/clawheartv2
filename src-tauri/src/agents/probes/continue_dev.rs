//! Continue.dev 配置探测器
//!
//! 配置路径：~/.continue/config.json
//! 配置结构：
//!   {
//!     "models": [
//!       { "title": "...", "provider": "openai", "apiBase": "...", "apiKey": "..." }
//!     ]
//!   }
//!
//! Probe 策略：在 models[] 数组顶部插入或更新一个 ClawHeart 路由模型。

use crate::agents::config_probe::*;
use crate::agents::DiscoveredAgent;
use std::path::PathBuf;

pub struct ContinueProbe;

const CLAWHEART_MODEL_TITLE: &str = "ClawHeart Route";

fn settings_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".continue/config.json"))
}

fn read_json(path: &std::path::Path) -> serde_json::Value {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| {
            serde_json::json!({ "models": [] })
        }),
        Err(_) => serde_json::json!({ "models": [] }),
    }
}

fn extract_first_base_url(json: &serde_json::Value) -> Option<String> {
    json.get("models")?.as_array()?.iter().find_map(|m| {
        m.get("apiBase")
            .and_then(|v| v.as_str())
            .map(String::from)
    })
}

impl ConfigProbe for ContinueProbe {
    fn platform(&self) -> &'static str { "continue" }

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
                current_base_url = extract_first_base_url(&json);
                current_key_present = json
                    .get("models")
                    .and_then(|m| m.as_array())
                    .map(|arr| arr.iter().any(|m| m.get("apiKey").and_then(|v| v.as_str()).map(|s| !s.is_empty()).unwrap_or(false)))
                    .unwrap_or(false);
            }
        }

        let mut warnings = Vec::new();
        if !exists {
            warnings.push("Continue 配置文件不存在".into());
        }

        ProbeResult {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "continue".into(),
            agent_name: agent.agent_name.clone(),
            current_base_url,
            current_key_present,
            config_source: ConfigSource::JsonFile {
                path: path_str,
                json_path: "models[0].apiBase".into(),
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
            serde_json::json!({ "models": [] })
        };
        let before = serde_json::to_string_pretty(&before_json)
            .unwrap_or_else(|_| "{}".into());

        let mut after_json = before_json.clone();
        let route_model = serde_json::json!({
            "title": CLAWHEART_MODEL_TITLE,
            "provider": "openai",
            "apiBase": target.base_url,
            "apiKey": target.virtual_key,
            "model": "auto",
        });

        // models 必须是数组；若不存在则创建
        if !after_json.is_object() {
            after_json = serde_json::Value::Object(serde_json::Map::new());
        }
        let models = after_json
            .as_object_mut()
            .unwrap()
            .entry("models".to_string())
            .or_insert(serde_json::Value::Array(vec![]));
        if let serde_json::Value::Array(arr) = models {
            // 移除旧 ClawHeart 项
            arr.retain(|m| {
                m.get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s != CLAWHEART_MODEL_TITLE)
                    .unwrap_or(true)
            });
            // 插入到顶部
            arr.insert(0, route_model);
        }
        let after = serde_json::to_string_pretty(&after_json)
            .unwrap_or_else(|_| "{}".into());

        let diff_lines = make_diff_lines(&before, &after);
        let risk = if !exists { PatchRisk::Caution } else { PatchRisk::Safe };

        Some(ConfigPatch {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "continue".into(),
            agent_name: agent.agent_name.clone(),
            source: ConfigSource::JsonFile {
                path: path.to_string_lossy().to_string(),
                json_path: "models[0]".into(),
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
            _ => return Err("Continue Probe 只支持 JsonFile 源".into()),
        };
        let write_path = if dry_run {
            dry_run_path("continue", "config.json")
        } else {
            real_path.clone()
        };
        ensure_parent(&write_path)?;
        std::fs::write(&write_path, &patch.after).map_err(|e| format!("写入失败：{}", e))?;
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
            dry_run_path("continue", "config.json")
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
        // 取第一个非 ClawHeart Route 的 model（避免读到自己回填的虚拟 key）
        let model = json
            .get("models")?
            .as_array()?
            .iter()
            .find(|m| {
                let title = m.get("title").and_then(|v| v.as_str()).unwrap_or("");
                title != CLAWHEART_MODEL_TITLE
            })?;
        let base_url = model.get("apiBase").and_then(|v| v.as_str()).map(String::from)?;
        let api_key = model.get("apiKey").and_then(|v| v.as_str()).map(String::from)?;
        if api_key.is_empty() {
            return None;
        }
        Some(CredentialReadResult {
            base_url,
            api_key,
            source_path: path.to_string_lossy().to_string(),
            source_label: "Continue.dev · config.json[models[0]]".into(),
        })
    }
}
