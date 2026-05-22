//! OpenClaw 配置探测器
//!
//! 配置路径：~/.openclaw/config.json （自家产品，最易接管）
//! 配置键：llm.base_url / llm.api_key

use crate::agents::config_probe::*;
use crate::agents::probes::ChannelCandidate;
use crate::agents::DiscoveredAgent;
use std::path::PathBuf;

// ──────────────────────────────────────────────────────────────────
// 反向导入 —— 解析 ~/.openclaw/openclaw.json 中 models.providers.* 多 provider 结构
// ──────────────────────────────────────────────────────────────────

/// 从 openclaw.json 提取所有 provider 为候选渠道列表
pub fn extract_channels(agent_id: &str) -> Vec<ChannelCandidate> {
    let Some(path) = home_dir().map(|h| h.join(".openclaw/openclaw.json")) else {
        return vec![];
    };
    let Ok(content) = std::fs::read_to_string(&path) else { return vec![]; };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else {
        return vec![];
    };

    let Some(providers) = v.pointer("/models/providers").and_then(|p| p.as_object()) else {
        return vec![];
    };

    // 收集 auth.profiles：识别"账号登录模式" 的 provider（无原生 api key）
    let auth_profiles: std::collections::HashSet<String> = v
        .pointer("/auth/profiles")
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.keys()
                .filter_map(|k| k.split(':').next().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut out = Vec::new();
    for (key, cfg) in providers {
        // baseUrl 字段 fallback：baseUrl / base_url / endpoint
        let base_url = cfg
            .get("baseUrl")
            .or_else(|| cfg.get("base_url"))
            .or_else(|| cfg.get("endpoint"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        if base_url.is_empty() { continue; }

        let api_key = cfg
            .get("apiKey")
            .or_else(|| cfg.get("api_key"))
            .or_else(|| cfg.get("token"))
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let default_model = cfg
            .get("models")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("id").or_else(|| m.get("name")))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        // 推断协议：用 `api` 字段标识 + base_url 启发式
        let api_type = cfg.get("api").and_then(|s| s.as_str()).unwrap_or("");
        let protocol = if api_type.contains("anthropic") || base_url.contains("/anthropic") {
            "anthropic"
        } else if api_type.contains("gemini") || base_url.contains("generativelanguage") {
            "gemini"
        } else {
            "openai"
        };

        let mut warnings = Vec::new();
        let is_account_managed = api_key.is_none() && auth_profiles.contains(key);
        if is_account_managed {
            warnings.push(
                "OpenClaw 账号登录模式：本机无原生 API key，需用户手动补一个直连 key 才能用 ClawHeart 代理"
                    .into(),
            );
        } else if api_key.is_none() {
            warnings.push("API Key 为空，导入后需手动配置".into());
        }
        if !base_url.starts_with("http") {
            warnings.push(format!("Base URL 格式可能不正确：{}", base_url));
        }

        out.push(ChannelCandidate {
            id: format!("openclaw:{}", key),
            name: key.clone(),
            source_agent_id: agent_id.to_string(),
            source_platform: "openclaw".to_string(),
            base_url,
            api_key,
            protocol: protocol.to_string(),
            default_model,
            provider_kind: "custom".to_string(),
            already_exists: false,
            warnings,
        });
    }
    out
}

pub struct OpenClawProbe;

const PATH_BASE_URL: &str = "llm.base_url";
const PATH_API_KEY: &str = "llm.api_key";

fn settings_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".openclaw/config.json"))
}

fn read_json(path: &std::path::Path) -> serde_json::Value {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| {
            serde_json::Value::Object(serde_json::Map::new())
        }),
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    }
}

impl ConfigProbe for OpenClawProbe {
    fn platform(&self) -> &'static str { "openclaw" }

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
            warnings.push("OpenClaw 配置文件不存在".into());
        }

        ProbeResult {
            agent_id: format!("{}/{}", agent.platform, agent.agent_name),
            agent_platform: "openclaw".into(),
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
            agent_platform: "openclaw".into(),
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
            _ => return Err("OpenClaw Probe 只支持 JsonFile 源".into()),
        };
        let write_path = if dry_run {
            dry_run_path("openclaw", "config.json")
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
            dry_run_path("openclaw", "config.json")
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
            source_label: format!("OpenClaw · config.json[{}]", PATH_BASE_URL),
        })
    }
}
