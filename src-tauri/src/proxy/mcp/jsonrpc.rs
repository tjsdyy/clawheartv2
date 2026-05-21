//! MCP JSON-RPC 2.0 类型定义
//!
//! MCP (Model Context Protocol) 协议层。所有消息都是 JSON-RPC 2.0：
//!   - Request：含 id、method、params；期待响应
//!   - Response：含 id、result 或 error
//!   - Notification：无 id；不期待响应（如 progress、log）
//!
//! MCP 关键方法：
//!   initialize / initialized
//!   tools/list / tools/call
//!   resources/list / resources/read
//!   prompts/list / prompts/get
//!   roots/list_changed
//!
//! 设计：保留 raw_json 字段，让 interceptor 既能用 typed 视图判断方法，又能透传未识别字段。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 标识 id：可能是 数字 / 字符串 / null
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RpcId {
    Number(i64),
    String(String),
    Null,
}

impl RpcId {
    pub fn as_string_key(&self) -> String {
        match self {
            RpcId::Number(n) => n.to_string(),
            RpcId::String(s) => s.clone(),
            RpcId::Null => "<null>".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn blocked(message: impl Into<String>) -> Self {
        Self {
            code: -32099, // ClawHeart 自定义：拦截
            message: message.into(),
            data: Some(Value::String("blocked_by_clawheart".into())),
        }
    }
}

/// 任一种 MCP 消息（用于按行解析）
#[derive(Debug, Clone)]
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
    Invalid(String, String), // (raw_line, parse_error)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: RpcId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: RpcId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

// ──────────────────────────────────────────────────────────────────
// 解析与序列化
// ──────────────────────────────────────────────────────────────────

/// 将一行 JSON 文本解析为 MCP 消息
///
/// MCP stdio 协议：每行一个 JSON 对象
pub fn parse_line(raw: &str) -> McpMessage {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return McpMessage::Invalid(raw.to_string(), "empty line".into());
    }
    let value: Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(e) => {
            return McpMessage::Invalid(raw.to_string(), format!("json parse: {}", e));
        }
    };

    // 三类判定：
    //   有 id + 有 method → Request
    //   有 id 但无 method → Response
    //   无 id 但有 method → Notification
    let has_id = value.get("id").is_some();
    let has_method = value.get("method").is_some();

    match (has_id, has_method) {
        (true, true) => match serde_json::from_value::<McpRequest>(value.clone()) {
            Ok(r) => McpMessage::Request(r),
            Err(e) => McpMessage::Invalid(raw.to_string(), format!("as request: {}", e)),
        },
        (true, false) => match serde_json::from_value::<McpResponse>(value.clone()) {
            Ok(r) => McpMessage::Response(r),
            Err(e) => McpMessage::Invalid(raw.to_string(), format!("as response: {}", e)),
        },
        (false, true) => match serde_json::from_value::<McpNotification>(value.clone()) {
            Ok(r) => McpMessage::Notification(r),
            Err(e) => McpMessage::Invalid(raw.to_string(), format!("as notification: {}", e)),
        },
        (false, false) => McpMessage::Invalid(
            raw.to_string(),
            "neither id nor method present".into(),
        ),
    }
}

pub fn serialize_message(msg: &McpMessage) -> Result<String, serde_json::Error> {
    match msg {
        McpMessage::Request(r) => serde_json::to_string(r),
        McpMessage::Response(r) => serde_json::to_string(r),
        McpMessage::Notification(r) => serde_json::to_string(r),
        McpMessage::Invalid(raw, _) => Ok(raw.clone()),
    }
}

/// 用于构造一条"被拦截"响应
pub fn build_error_response(id: RpcId, error: RpcError) -> McpResponse {
    McpResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(error),
    }
}

// ──────────────────────────────────────────────────────────────────
// 高级访问
// ──────────────────────────────────────────────────────────────────

impl McpRequest {
    /// 从 tools/call 请求中提取工具名
    pub fn tool_call_name(&self) -> Option<&str> {
        if self.method != "tools/call" {
            return None;
        }
        self.params
            .as_ref()?
            .get("name")
            .and_then(|v| v.as_str())
    }

    /// 从 tools/call 请求中提取 arguments（JSON 字符串化便于扫描）
    pub fn tool_call_arguments_text(&self) -> String {
        match &self.params {
            Some(p) => p
                .get("arguments")
                .map(|v| v.to_string())
                .unwrap_or_default(),
            None => String::new(),
        }
    }

    /// 从 resources/read 提取 URI
    pub fn resource_uri(&self) -> Option<&str> {
        if self.method != "resources/read" {
            return None;
        }
        self.params.as_ref()?.get("uri").and_then(|v| v.as_str())
    }
}

impl McpResponse {
    /// 从 tools/call 响应中提取所有 text 内容（拼接）
    pub fn tool_call_text_content(&self) -> String {
        let Some(result) = &self.result else {
            return String::new();
        };
        let Some(arr) = result.get("content").and_then(|v| v.as_array()) else {
            return String::new();
        };
        let mut buf = String::new();
        for item in arr {
            if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(t);
            }
        }
        buf
    }

    /// tools/list 响应：迭代所有工具的 (name, description)
    pub fn tools_list_entries(&self) -> Vec<(String, String)> {
        let Some(result) = &self.result else { return vec![]; };
        let Some(arr) = result.get("tools").and_then(|v| v.as_array()) else {
            return vec![];
        };
        arr.iter()
            .filter_map(|t| {
                let name = t.get("name").and_then(|v| v.as_str())?.to_string();
                let desc = t
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Some((name, desc))
            })
            .collect()
    }
}

// ──────────────────────────────────────────────────────────────────
// 单元测试
// ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_with_number_id() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let m = parse_line(raw);
        match m {
            McpMessage::Request(r) => {
                assert_eq!(r.method, "tools/list");
                assert_eq!(r.id, RpcId::Number(1));
            }
            _ => panic!("expected Request"),
        }
    }

    #[test]
    fn parse_tools_call_extract_name() {
        let raw = r#"{"jsonrpc":"2.0","id":"abc","method":"tools/call","params":{"name":"fs.read","arguments":{"path":"/etc/passwd"}}}"#;
        let m = parse_line(raw);
        match m {
            McpMessage::Request(r) => {
                assert_eq!(r.tool_call_name(), Some("fs.read"));
                assert!(r.tool_call_arguments_text().contains("/etc/passwd"));
            }
            _ => panic!("expected Request"),
        }
    }

    #[test]
    fn parse_response_with_text_content() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"hello"},{"type":"text","text":"world"}]}}"#;
        let m = parse_line(raw);
        match m {
            McpMessage::Response(r) => {
                assert_eq!(r.tool_call_text_content(), "hello\nworld");
            }
            _ => panic!("expected Response"),
        }
    }

    #[test]
    fn parse_notification() {
        let raw = r#"{"jsonrpc":"2.0","method":"notifications/progress","params":{"progress":0.5}}"#;
        let m = parse_line(raw);
        matches!(m, McpMessage::Notification(_));
    }

    #[test]
    fn parse_invalid_json() {
        let m = parse_line("{not json");
        match m {
            McpMessage::Invalid(_, _) => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn parse_tools_list_entries() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"fs.read","description":"Read file"},{"name":"net.fetch","description":"HTTP GET"}]}}"#;
        let m = parse_line(raw);
        match m {
            McpMessage::Response(r) => {
                let entries = r.tools_list_entries();
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "fs.read");
                assert_eq!(entries[1].1, "HTTP GET");
            }
            _ => panic!("expected Response"),
        }
    }

    #[test]
    fn build_error_response_works() {
        let resp = build_error_response(RpcId::Number(7), RpcError::blocked("DLP hit"));
        let serialized = serde_json::to_string(&resp).unwrap();
        assert!(serialized.contains("DLP hit"));
        assert!(serialized.contains("-32099"));
    }
}
