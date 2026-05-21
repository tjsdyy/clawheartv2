//! Row 结构 — 与 schema 1:1 对应

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub system_status: String,
    pub user_enabled: bool,
    pub safety_label: String,
    pub scan_score: i32,
    pub install_path: Option<String>,
    pub installed_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptEvent {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub severity: String,
    pub signal_class: String,
    pub rule_id: Option<String>,
    pub mitre_attack_id: Option<String>,
    pub confidence: String,
    pub details: String,
    pub evidence: Option<String>,
    pub prompt_snippet: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: i64,
    pub timestamp: String,
    pub agent_id: Option<String>,
    pub format: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub blocked: bool,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub latency_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub id: i64,
    pub platform: String,
    pub agent_name: String,
    pub config_path: Option<String>,
    pub process_name: Option<String>,
    pub last_seen: String,
    pub mcp_servers: Option<String>,
    pub config_hash: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub affected: String,
    pub cvss_score: Option<f64>,
    pub action: Option<String>,
    pub published: String,
    pub fetched_at: String,
    pub dismissed: bool,
}
