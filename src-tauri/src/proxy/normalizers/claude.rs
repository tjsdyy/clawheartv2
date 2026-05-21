//! Anthropic Messages API normalizer

use crate::proxy::formats::LlmFormat;
use crate::proxy::normalizer::*;
use serde_json::Value;

pub struct ClaudeNormalizer;

impl RequestNormalizer for ClaudeNormalizer {
    fn format(&self) -> LlmFormat { LlmFormat::Claude }

    fn normalize(&self, raw: &Value, _headers: &http_like::HeaderMap) -> Result<NormalizedRequest, NormalizeError> {
        let model = raw.get("model").and_then(|v| v.as_str())
            .ok_or(NormalizeError::MissingField("model"))?.to_string();

        let messages = raw.get("messages").and_then(|m| m.as_array())
            .ok_or(NormalizeError::MissingField("messages"))?;

        let mut norm_messages = Vec::new();

        // Claude 还有 top-level "system" 字段
        if let Some(system) = raw.get("system").and_then(|s| s.as_str()) {
            norm_messages.push(NormMessage {
                role: Role::System,
                text_content: system.to_string(),
                tool_calls: vec![], tool_results: vec![],
            });
        }

        for msg in messages {
            let role_str = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let role = match role_str {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                _ => Role::User,
            };

            let mut text = String::new();
            let mut tool_calls = Vec::new();
            let mut tool_results = Vec::new();

            // Claude content 可以是 string 或 content blocks 数组
            match msg.get("content") {
                Some(Value::String(s)) => text = s.clone(),
                Some(Value::Array(arr)) => {
                    for block in arr {
                        match block.get("type").and_then(|t| t.as_str()) {
                            Some("text") => {
                                if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                                    if !text.is_empty() { text.push('\n'); }
                                    text.push_str(t);
                                }
                            }
                            Some("tool_use") => {
                                tool_calls.push(ToolCall {
                                    id: block.get("id").and_then(|v| v.as_str()).unwrap_or("").into(),
                                    name: block.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                                    arguments: block.get("input").cloned().unwrap_or(Value::Null),
                                });
                            }
                            Some("tool_result") => {
                                tool_results.push(ToolResult {
                                    call_id: block.get("tool_use_id").and_then(|v| v.as_str()).unwrap_or("").into(),
                                    content: block.get("content").and_then(|v| v.as_str()).unwrap_or("").into(),
                                    is_error: block.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }

            norm_messages.push(NormMessage {
                role, text_content: text, tool_calls, tool_results,
            });
        }

        Ok(NormalizedRequest {
            model,
            messages: norm_messages,
            tools: vec![], // TODO: parse top-level "tools"
            stream: raw.get("stream").and_then(|s| s.as_bool()).unwrap_or(false),
            max_tokens: raw.get("max_tokens").and_then(|v| v.as_u64()).map(|n| n as u32),
            temperature: raw.get("temperature").and_then(|v| v.as_f64()),
            original_format: LlmFormat::Claude,
            original_body: raw.clone(),
        })
    }

    fn denormalize(&self, norm: &NormalizedRequest) -> Result<Value, NormalizeError> {
        Ok(norm.original_body.clone())
    }
}
