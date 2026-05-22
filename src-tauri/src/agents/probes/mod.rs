//! Per-platform ConfigProbe 实现
//!
//! 设计：[`docs/proposals/agent-config-autoapply.md`] §3

pub mod claude_code;
pub mod codex_cli;
pub mod continue_dev;
pub mod cursor;
pub mod gemini_cli;
pub mod hermes;
pub mod opencode;
pub mod openclaw;
pub mod openeva;

use serde::{Deserialize, Serialize};

/// 反向导入候选渠道（共享类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCandidate {
    pub id: String,
    pub name: String,
    pub source_agent_id: String,
    pub source_platform: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub protocol: String,
    pub default_model: Option<String>,
    pub provider_kind: String,
    pub already_exists: bool,
    pub warnings: Vec<String>,
}

/// 支持反向导入的平台列表（前端用于决定是否显示导入入口）
pub const IMPORTABLE_PLATFORMS: &[&str] = &[
    "openclaw",
    "openeva",
    "opencode",
    "hermes",
    "claude",
    "codex",
    "gemini",
];
