//! Agent 发现 + 文件漂移监控

pub mod config_probe;
pub mod drift;
pub mod platforms;
pub mod probes;
pub mod process;
pub mod scanner;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub platform: String,
    pub agent_name: String,
    pub config_path: Option<String>,
    pub process_name: Option<String>,
    pub last_seen: String,
    pub mcp_servers: Vec<String>,
    pub config_hash: Option<String>,
    /// 状态：active | config_broken | candidate | idle | offline
    pub status: String,
    /// 未知平台候选发现时记录的命中线索（如 ["skills/", "mcp.json", "*.json:apiKey"]）
    /// 已知平台默认为空。仅用于 UI 展示，不持久化。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discovery_signals: Vec<String>,
}
