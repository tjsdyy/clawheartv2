//! 流式扫描状态机 — 9Router 借鉴
//!
//! SSE 响应是 chunk by chunk 流过来；不能等全部收齐再扫（延迟 + 内存）。
//! 用一个状态机增量地累积、增量地匹配；遇到匹配立即 block + 注入 LLM 格式 block 响应。

use serde::Serialize;

#[derive(Debug, Default)]
pub struct StreamScanState {
    buffer: String,
    /// 已扫到的偏移；下次只扫 buffer[scanned..]
    scanned: usize,
    /// 最大保留窗口（避免长流耗尽内存）
    max_buffer: usize,
}

impl StreamScanState {
    pub fn new() -> Self {
        Self { buffer: String::new(), scanned: 0, max_buffer: 64 * 1024 }
    }

    pub fn push_chunk(&mut self, chunk: &str) -> ChunkVerdict {
        self.buffer.push_str(chunk);

        // 简单：每次 push 都扫整个 buffer 的 DLP（W11 升级为流式 trie）
        let r = crate::security::redact::redact(&self.buffer);
        if !r.hits.is_empty() {
            return ChunkVerdict::Redact { placeholders: r.hits.iter().map(|h| h.placeholder.clone()).collect() };
        }

        let injection = crate::security::injection::scan(&self.buffer[self.scanned..]);
        if !injection.is_empty() {
            return ChunkVerdict::Block {
                reason: format!("Injection in response stream: {}", injection[0].pattern_id),
            };
        }

        self.scanned = self.buffer.len();

        // GC 超过窗口的前缀（仅保留尾部以便上下文）
        if self.buffer.len() > self.max_buffer {
            let drop = self.buffer.len() - self.max_buffer;
            self.buffer.drain(..drop);
            self.scanned = self.scanned.saturating_sub(drop);
        }

        ChunkVerdict::Pass
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ChunkVerdict {
    Pass,
    Redact { placeholders: Vec<String> },
    Block { reason: String },
}
