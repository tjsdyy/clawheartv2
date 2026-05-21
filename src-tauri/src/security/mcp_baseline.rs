//! MCP 工具基线冻结 — 防 "tool rug-pull"
//!
//! 会话开始时把所有 tools/list 响应记录到 mcp_tool_baselines 表
//! （description hash + capability 推断）。
//!
//! 每次 tools/call 比对当前 server 的 tool 描述哈希；不一致 → mcp_tool_drift 事件。

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ToolBaselineEntry {
    pub session_id: String,
    pub server_id: String,
    pub tool_name: String,
    pub description_hash: String,
    pub capability: Capability,
    pub frozen_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Read,
    Write,
    Exec,
    Network,
    Credentials,
    Unknown,
}

/// 从 tool description 推断 capability（W11 升级为 NLP 分类器）。
pub fn infer_capability(description: &str) -> Capability {
    let d = description.to_lowercase();
    if d.contains("execute") || d.contains("run") || d.contains("shell") { return Capability::Exec; }
    if d.contains("write") || d.contains("modify") || d.contains("delete") { return Capability::Write; }
    if d.contains("read") || d.contains("get") || d.contains("list") { return Capability::Read; }
    if d.contains("post") || d.contains("fetch") || d.contains("http") || d.contains("network") { return Capability::Network; }
    if d.contains("credential") || d.contains("secret") || d.contains("token") { return Capability::Credentials; }
    Capability::Unknown
}

/// SHA-256 摘要（基于本仓库 `security::sha256` 自实现）。
pub fn hash_description(s: &str) -> String {
    crate::security::sha256::hex_string(s)
}

#[derive(Default)]
pub struct ToolBaseline {
    /// key = (session_id, server_id, tool_name)
    entries: HashMap<(String, String, String), String>,
}

impl ToolBaseline {
    pub fn new() -> Self { Self::default() }

    /// 会话开始时记录基线。
    pub fn freeze(&mut self, session: &str, server: &str, tool: &str, description: &str) {
        let key = (session.to_string(), server.to_string(), tool.to_string());
        self.entries.insert(key, hash_description(description));
    }

    /// 调用时比对；返回是否漂移。
    pub fn check_drift(&self, session: &str, server: &str, tool: &str, current_description: &str) -> bool {
        let key = (session.to_string(), server.to_string(), tool.to_string());
        if let Some(baseline_hash) = self.entries.get(&key) {
            return baseline_hash != &hash_description(current_description);
        }
        false // 未冻结过 → 不视为漂移
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_capabilities() {
        assert_eq!(infer_capability("List files in a directory"), Capability::Read);
        assert_eq!(infer_capability("Execute a shell command"), Capability::Exec);
        assert_eq!(infer_capability("Post data to URL"), Capability::Network);
    }

    #[test]
    fn detects_drift() {
        let mut b = ToolBaseline::new();
        b.freeze("s1", "fs", "read", "Read a file");
        assert!(!b.check_drift("s1", "fs", "read", "Read a file"));
        assert!(b.check_drift("s1", "fs", "read", "Read a file (and exfiltrate)"));
    }
}
