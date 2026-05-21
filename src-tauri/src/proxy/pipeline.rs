//! 请求管线 — 串联 L0 ~ L4，输出 PipelineVerdict
//!
//! 调用顺序：
//!   L0 detect_format → normalize → L7 KS snapshot → L2 内容检查 →
//!   L3 MCP（如果是 MCP 请求） → L1 熔断器选上游 → 转发

use crate::proxy::formats::LlmFormat;
use crate::proxy::normalizer::NormalizedRequest;
use crate::security::{
    danger, injection, redact,
    kill_switch::KillSwitch,
    signal::{classify, SignalClass},
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub enum PipelineVerdict {
    Allow {
        normalized: NormalizedRequest,
    },
    Block {
        reason: String,
        rule_id: String,
        mitre_attack_id: Option<String>,
        signal_class: SignalClass,
    },
    Warn {
        normalized: NormalizedRequest,
        notes: Vec<String>,
    },
}

pub struct Pipeline {
    pub kill_switch: Arc<KillSwitch>,
}

impl Pipeline {
    pub fn new(kill_switch: Arc<KillSwitch>) -> Self {
        Self { kill_switch }
    }

    pub fn process(&self, _path: &str, _body: &serde_json::Value, normalized: NormalizedRequest) -> PipelineVerdict {
        // L7 Kill Switch（必须最先）
        if self.kill_switch.snapshot() {
            return PipelineVerdict::Block {
                reason: "Kill Switch active".into(),
                rule_id: "KS-001".into(),
                mitre_attack_id: None,
                signal_class: classify("manual_block"),
            };
        }

        // L2 内容层 — 危险指令
        let text = crate::proxy::normalizer::collect_all_text(&normalized);
        let danger_hits = danger::scan(&text);
        if let Some(hit) = danger_hits.first() {
            return PipelineVerdict::Block {
                reason: format!("Danger command: {}", hit.description),
                rule_id: hit.rule_id.clone(),
                mitre_attack_id: hit.mitre_attack_id.clone(),
                signal_class: classify("danger_command"),
            };
        }

        // L2 — 提示词注入
        let inj_hits = injection::scan(&text);
        if let Some(hit) = inj_hits.first() {
            // 注入默认 warn（不阻断对话，仅记录 + 通知）；W11 提供配置可强制 block
            let mut notes = vec![format!("Injection pattern {}: {}", hit.pattern_id, hit.matched_needle)];
            for h in &inj_hits[1..] {
                notes.push(format!("Injection pattern {}: {}", h.pattern_id, h.matched_needle));
            }
            return PipelineVerdict::Warn { normalized, notes };
        }

        // L2 — 凭据 DLP
        let redact_result = redact::redact(&text);
        if !redact_result.hits.is_empty() {
            return PipelineVerdict::Block {
                reason: format!("Credential leak: {}", redact_result.hits[0].class),
                rule_id: redact_result.hits[0].pattern_id.clone(),
                mitre_attack_id: Some("T1552".into()),
                signal_class: classify("credential_leak"),
            };
        }

        // L3 MCP — TODO: 接入 mcp_chain 检测器 + baseline；在 W11 实现
        // L4 供应链 — TODO

        PipelineVerdict::Allow { normalized }
    }
}

#[allow(dead_code)]
fn _format_signal_check(format: LlmFormat) {
    let _ = format;
}
