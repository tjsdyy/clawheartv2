//! 信号分类 — Pipelock 心智：把"告警"分成 4 类，让运维/SIEM 区分。
//!
//! - Threat：真正的攻击行为（注入、攻击链、外泄）
//! - Protective：用户自定义的规则触发（如预算上限、自定义 deny list）
//! - ConfigMismatch：配置漂移（MCP 工具描述变化、技能漂移、Agent 配置变更）
//! - InfraError：基础设施异常（feed 签名失败、上游 TLS 错误、磁盘满）

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalClass {
    Threat,
    Protective,
    ConfigMismatch,
    InfraError,
}

impl SignalClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalClass::Threat => "Threat",
            SignalClass::Protective => "Protective",
            SignalClass::ConfigMismatch => "ConfigMismatch",
            SignalClass::InfraError => "InfraError",
        }
    }
}

/// 从事件类型推断分类（IPC 事件入库前调用）。
pub fn classify(event_type: &str) -> SignalClass {
    match event_type {
        "injection" | "mcp_injection" | "mcp_chain" | "credential_leak" | "danger_command" | "exfiltration" =>
            SignalClass::Threat,
        "budget_exceeded" | "skill_disabled" | "manual_block" =>
            SignalClass::Protective,
        "mcp_drift" | "skill_drift" | "agent_drift" | "file_drift" | "config_change" =>
            SignalClass::ConfigMismatch,
        "feed_signature_failed" | "upstream_tls_error" | "disk_full" | "ca_not_trusted" =>
            SignalClass::InfraError,
        _ => SignalClass::Threat, // 默认走 Threat（失败关闭哲学）
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_correctly() {
        assert_eq!(classify("mcp_chain"), SignalClass::Threat);
        assert_eq!(classify("budget_exceeded"), SignalClass::Protective);
        assert_eq!(classify("mcp_drift"), SignalClass::ConfigMismatch);
        assert_eq!(classify("feed_signature_failed"), SignalClass::InfraError);
        assert_eq!(classify("unknown_event"), SignalClass::Threat);
    }
}
