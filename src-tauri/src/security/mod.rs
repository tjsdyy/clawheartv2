//! 安全引擎门面 — SecurityEngine
//!
//! 所有安全检查统一通过 [`SecurityCheck`] trait；
//! Pipeline 在 `proxy/pipeline.rs` 串联 L2/L3/L4。
//!
//! 全局不变量：**任一检查 panic/timeout → 走 block 路径（失败关闭）**。

pub mod advisory;
pub mod budget;
pub mod checks;
pub mod danger;
pub mod injection;
pub mod kill_switch;
pub mod mcp;
pub mod mcp_baseline;
pub mod mcp_chains;
pub mod mcp_check;
pub mod normalizer;
pub mod redact;
pub mod rule_registry;
pub mod scanner;
pub mod sha256;
pub mod signal;
pub mod skill_scanner;
pub mod skills;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReason {
    pub code: String,
    pub message: String,
    pub mitre_attack_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckOutcome {
    Pass,
    Block {
        reason: BlockReason,
        fix_hint: Option<String>,
    },
    Warn {
        detail: String,
    },
    /// 由后续 chunk-level 扫描决定（流式响应专用）
    Defer,
}

impl CheckOutcome {
    pub fn is_block(&self) -> bool {
        matches!(self, CheckOutcome::Block { .. })
    }
}

/// 安全检查的统一接口；所有检查必须实现该 trait。
pub trait SecurityCheck: Send + Sync {
    fn id(&self) -> &str;
    fn severity(&self) -> Severity;
    fn check(&self, ctx: &CheckContext<'_>) -> CheckOutcome;
}

pub struct CheckContext<'a> {
    pub text: &'a str,
    pub agent_id: Option<&'a str>,
    pub session_id: Option<&'a str>,
    pub skill_headers: &'a [String],
}
