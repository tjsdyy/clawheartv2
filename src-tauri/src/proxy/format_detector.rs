//! 协议格式自动检测 — 9Router 借鉴
//!
//! 不维护 800+ 模型枚举；只维护 ~30 条 host→format 映射 +
//! URL 路径强信号 + body 结构启发式。

use super::formats::LlmFormat;
use serde_json::Value;

pub fn detect_format(path: &str, body: &Value) -> LlmFormat {
    // 1) URL 路径强信号
    if path.contains("/v1/messages")  { return LlmFormat::Claude; }
    if path.contains("/v1/responses") { return LlmFormat::OpenAIResponses; }
    if path.contains("/api/chat")     { return LlmFormat::Ollama; }
    if path.contains("generativelanguage.googleapis.com")
        || path.contains(":generateContent")
        || path.contains(":streamGenerateContent") {
        return LlmFormat::Gemini;
    }

    // 2) body 结构启发式
    if body.get("contents").is_some() { return LlmFormat::Gemini; }
    if body.get("input").is_some() && body.get("messages").is_none() {
        return LlmFormat::OpenAIResponses;
    }
    if let Some(arr) = body.get("messages").and_then(|m| m.as_array()) {
        if arr.iter().any(has_claude_content_blocks) { return LlmFormat::Claude; }
    }
    if body.get("messages").is_some() { return LlmFormat::OpenAI; }

    LlmFormat::Unknown
}

fn has_claude_content_blocks(msg: &Value) -> bool {
    msg.get("content")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter().any(|item| {
                item.get("type").and_then(|t| t.as_str())
                    .is_some_and(|t| matches!(t, "text" | "image" | "tool_use" | "tool_result"))
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_claude_by_path() {
        let body = json!({});
        assert_eq!(detect_format("/v1/messages", &body), LlmFormat::Claude);
    }

    #[test]
    fn detects_responses_by_input_field() {
        let body = json!({ "input": "Hello", "model": "gpt-5.4" });
        assert_eq!(detect_format("/anything", &body), LlmFormat::OpenAIResponses);
    }

    #[test]
    fn detects_gemini_by_contents_field() {
        let body = json!({ "contents": [{"role": "user", "parts": [{"text": "hi"}]}] });
        assert_eq!(detect_format("/anything", &body), LlmFormat::Gemini);
    }

    #[test]
    fn detects_claude_by_content_blocks() {
        let body = json!({
            "messages": [{"role":"user","content":[{"type":"text","text":"hi"}]}]
        });
        assert_eq!(detect_format("/anything", &body), LlmFormat::Claude);
    }

    #[test]
    fn detects_openai_default() {
        let body = json!({
            "model": "gpt-4",
            "messages": [{"role":"user","content":"hi"}]
        });
        assert_eq!(detect_format("/v1/chat/completions", &body), LlmFormat::OpenAI);
    }
}
