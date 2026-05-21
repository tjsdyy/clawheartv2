//! ConfigProbe trait —— Agent 配置探测与覆盖
//!
//! 设计：[`docs/proposals/agent-config-autoapply.md`]
//!
//! W7 安全策略：
//! - 默认所有写入走 dry-run 沙箱目录（~/.clawheart-v2/dry-run/<platform>/）
//! - 真实写入目标 Agent 配置文件需启用 `apply_real` feature（W8 启用后开放）
//! - 每次 apply 前必读取原文件并存 snapshot
//! - rollback 直接从 snapshot.before_value 恢复

use crate::agents::DiscoveredAgent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigSource {
    JsonFile {
        path: String,
        json_path: String,
    },
    TomlFile {
        path: String,
        key: String,
    },
    EnvVar {
        name: String,
        scope: String, // "shell_rc" | "launchd" | "user_env"
    },
    VsCodeWorkspace {
        path: String,
        setting: String,
    },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatchRisk {
    Safe,
    Caution,
    Risky,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub agent_id: String,
    pub agent_platform: String,
    pub agent_name: String,
    pub current_base_url: Option<String>,
    pub current_key_present: bool,
    pub config_source: ConfigSource,
    pub writable: bool,
    pub probe_available: bool, // false = 该平台 Probe 尚未实现
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPatch {
    pub agent_id: String,
    pub agent_platform: String,
    pub agent_name: String,
    pub source: ConfigSource,
    pub before: String,
    pub after: String,
    pub diff_lines: Vec<DiffLine>,
    pub risk_level: PatchRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: String, // "-" | "+" | " "
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyOutcome {
    pub agent_id: String,
    pub success: bool,
    pub snapshot_id: Option<String>,
    pub config_path: String,
    pub message: String,
    /// dry_run = true 表示仅写入沙箱目录；false 表示写入真实位置
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct OverwriteTarget<'a> {
    pub base_url: &'a str,
    pub virtual_key: &'a str,
    pub protocol: &'a str,
    pub profile_id: &'a str,
}

pub trait ConfigProbe: Send + Sync {
    fn platform(&self) -> &'static str;
    fn inspect(&self, agent: &DiscoveredAgent) -> ProbeResult;
    fn plan_overwrite(
        &self,
        agent: &DiscoveredAgent,
        target: &OverwriteTarget<'_>,
    ) -> Option<ConfigPatch>;

    /// 真实写入（dry_run = true 时写沙箱目录）。返回 (config_path, before, after)
    fn apply(
        &self,
        patch: &ConfigPatch,
        dry_run: bool,
    ) -> Result<AppliedPatch, String>;

    /// 从 snapshot.before_value 恢复（dry_run = true 时只校验逻辑可行性）
    fn rollback(
        &self,
        config_path: &str,
        before_value: &str,
        dry_run: bool,
    ) -> Result<(), String>;

    /// 仅 import 流程调用：读取真实 base_url + api_key（用于从 Agent 配置中导入 Profile）。
    /// 返回的 CredentialReadResult 在调用栈内被立即消费，永不持久化到磁盘。
    fn inspect_with_credential(
        &self,
        agent: &DiscoveredAgent,
    ) -> Option<CredentialReadResult>;
}

pub struct AppliedPatch {
    pub config_path: String,
    pub before_value: String,
    pub after_value: String,
    pub dry_run: bool,
}

/// inspect_with_credential 的返回值：仅在 import 流程内部短暂存在
pub struct CredentialReadResult {
    pub base_url: String,
    pub api_key: String,
    pub source_path: String,
    pub source_label: String, // 显示用："Cursor · settings.json[cursor.openaiApiKey]"
}

// ──────────────────────────────────────────────────────────────────
// Probe Registry
// ──────────────────────────────────────────────────────────────────

pub fn probe_for(platform: &str) -> Option<Box<dyn ConfigProbe>> {
    match platform {
        "cursor" => Some(Box::new(crate::agents::probes::cursor::CursorProbe)),
        "claude" => Some(Box::new(crate::agents::probes::claude_code::ClaudeCodeProbe)),
        "continue" => Some(Box::new(crate::agents::probes::continue_dev::ContinueProbe)),
        "codex" => Some(Box::new(crate::agents::probes::codex_cli::CodexCliProbe)),
        "openclaw" => Some(Box::new(crate::agents::probes::openclaw::OpenClawProbe)),
        _ => None,
    }
}

pub fn supported_platforms() -> &'static [&'static str] {
    &["cursor", "claude", "continue", "codex", "openclaw"]
}

// ──────────────────────────────────────────────────────────────────
// Common helpers (Probe 实现共用)
// ──────────────────────────────────────────────────────────────────

pub fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
}

pub fn dry_run_path(platform: &str, relative: &str) -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".clawheart-v2")
        .join("dry-run")
        .join(platform)
        .join(relative)
}

pub fn ensure_parent(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败：{}", e))?;
    }
    Ok(())
}

/// 生成 unified diff 行（简化版，按行对比）
pub fn make_diff_lines(before: &str, after: &str) -> Vec<DiffLine> {
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    let mut out = Vec::new();
    let max = before_lines.len().max(after_lines.len());
    for i in 0..max {
        match (before_lines.get(i), after_lines.get(i)) {
            (Some(b), Some(a)) if b == a => {
                if out.len() < 16 {
                    out.push(DiffLine {
                        kind: " ".into(),
                        text: (*b).to_string(),
                    });
                }
            }
            (Some(b), Some(a)) => {
                out.push(DiffLine { kind: "-".into(), text: (*b).to_string() });
                out.push(DiffLine { kind: "+".into(), text: (*a).to_string() });
            }
            (Some(b), None) => {
                out.push(DiffLine { kind: "-".into(), text: (*b).to_string() });
            }
            (None, Some(a)) => {
                out.push(DiffLine { kind: "+".into(), text: (*a).to_string() });
            }
            (None, None) => {}
        }
    }
    out
}

/// 在 JSON 对象的指定路径设置值（点号分隔，仅支持顶层 / 一级嵌套）
pub fn json_set_path(
    json: &mut serde_json::Value,
    path: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut cursor = json;
    for (i, key) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let serde_json::Value::Object(map) = cursor {
                map.insert((*key).to_string(), value);
                return Ok(());
            } else {
                return Err(format!("路径 {} 非对象", path));
            }
        }
        if let serde_json::Value::Object(map) = cursor {
            cursor = map
                .entry((*key).to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
        } else {
            return Err(format!("路径 {} 中间节点非对象", path));
        }
    }
    Ok(())
}

pub fn json_get_path(json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut cursor = json;
    for key in &parts {
        cursor = cursor.get(key)?;
    }
    Some(cursor.clone())
}
