//! CLI 输出统一封装。
//!
//! JSON 模式产出顶层固定结构 `{ ok, data?, error? }`，给 AI Agent 直接解析。
//! Human 模式按 stdout 友好打印（带 ANSI 色），同时把同样的语义信息塞给用户。

use serde::Serialize;
use serde_json::Value;

/// 统一输出 wrapper
pub struct Output {
    pub ok: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
    /// Human 模式渲染用的纯文本（JSON 模式忽略）
    pub human_text: Option<String>,
}

impl Output {
    pub fn ok<T: Serialize>(data: T) -> Self {
        Self {
            ok: true,
            data: serde_json::to_value(&data).ok(),
            error: None,
            human_text: None,
        }
    }

    pub fn ok_with_text<T: Serialize>(data: T, human_text: impl Into<String>) -> Self {
        Self {
            ok: true,
            data: serde_json::to_value(&data).ok(),
            error: None,
            human_text: Some(human_text.into()),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(msg.into()),
            human_text: None,
        }
    }

    /// 写到 stdout：JSON 模式 → 一行 JSON；Human 模式 → human_text 优先，回退到 data 的 Debug
    pub fn emit(self, json: bool) {
        if json {
            let v = serde_json::json!({
                "ok": self.ok,
                "data": self.data,
                "error": self.error,
            });
            // 用 to_string 保证单行（便于 stream 解析），人类调试可加 --json -v 重定向到 jq
            println!("{}", v);
        } else if self.ok {
            if let Some(text) = self.human_text {
                println!("{}", text);
            } else if let Some(data) = self.data {
                // 回退：pretty JSON
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            }
        } else {
            eprintln!(
                "{}",
                self.error
                    .unwrap_or_else(|| "未知错误".into())
            );
        }
    }
}

/// Result 简写，CLI 内部统一用 String 错误。
pub type CliResult = Result<(), String>;
