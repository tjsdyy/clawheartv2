//! Google Gemini normalizer
use crate::proxy::formats::LlmFormat;
use crate::proxy::normalizer::*;
use serde_json::Value;

pub struct GeminiNormalizer;

impl RequestNormalizer for GeminiNormalizer {
    fn format(&self) -> LlmFormat { LlmFormat::Gemini }

    fn normalize(&self, raw: &Value, _headers: &http_like::HeaderMap) -> Result<NormalizedRequest, NormalizeError> {
        // Gemini 的 model 通常在 URL 而非 body；这里从 generationConfig 或 fallback
        let model = raw.get("model").and_then(|v| v.as_str()).unwrap_or("gemini-2.5-pro").to_string();

        let contents = raw.get("contents").and_then(|c| c.as_array())
            .ok_or(NormalizeError::MissingField("contents"))?;

        let mut messages = Vec::new();
        for c in contents {
            let role_str = c.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let role = match role_str {
                "user" => Role::User,
                "model" => Role::Assistant,
                _ => Role::User,
            };
            let parts = c.get("parts").and_then(|p| p.as_array()).cloned().unwrap_or_default();
            let mut text = String::new();
            for p in parts {
                if let Some(t) = p.get("text").and_then(|v| v.as_str()) {
                    if !text.is_empty() { text.push('\n'); }
                    text.push_str(t);
                }
            }
            messages.push(NormMessage {
                role, text_content: text, tool_calls: vec![], tool_results: vec![],
            });
        }

        Ok(NormalizedRequest {
            model, messages, tools: vec![],
            stream: false,
            max_tokens: raw.pointer("/generationConfig/maxOutputTokens").and_then(|v| v.as_u64()).map(|n| n as u32),
            temperature: raw.pointer("/generationConfig/temperature").and_then(|v| v.as_f64()),
            original_format: LlmFormat::Gemini,
            original_body: raw.clone(),
        })
    }

    fn denormalize(&self, norm: &NormalizedRequest) -> Result<Value, NormalizeError> {
        Ok(norm.original_body.clone())
    }
}
