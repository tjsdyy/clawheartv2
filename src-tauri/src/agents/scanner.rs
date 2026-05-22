//! Agent 发现调度器
//!
//! 流程：每 60s tokio interval → 所有 PlatformScanner 并发 scan →
//! 合并结果到 discovered_agents 表 → 触发 ch://event/agent-discovered

use super::platforms::PlatformScanner;
use super::DiscoveredAgent;
use std::sync::Arc;

pub struct Scanner {
    platforms: Vec<Arc<dyn PlatformScanner>>,
}

impl Scanner {
    pub fn with_default_platforms() -> Self {
        Self {
            platforms: vec![
                Arc::new(super::platforms::claude::ClaudePlatform),
                Arc::new(super::platforms::codex::CodexPlatform),
                Arc::new(super::platforms::cursor::CursorPlatform),
                Arc::new(super::platforms::gemini::GeminiPlatform),
                Arc::new(super::platforms::windsurf::WindsurfPlatform),
                Arc::new(super::platforms::openclaw::OpenClawPlatform),
                Arc::new(super::platforms::openeva::OpenEvaPlatform),
                Arc::new(super::platforms::hermes::HermesPlatform),
                // 未知平台候选探测 —— 扫 ~/.<name>/ 含 AI 线索的目录，
                // 返回 status: "candidate"，UI 引导用户确认。
                Arc::new(super::platforms::unknown::UnknownPlatform),
            ],
        }
    }

    /// 串行 scan 所有平台（W17 改为 join_all 并发）。
    pub fn scan_once(&self) -> Vec<DiscoveredAgent> {
        let mut all = Vec::new();
        for p in &self.platforms {
            match p.scan() {
                Ok(items) => all.extend(items),
                Err(e) => tracing::warn!("platform scanner {} failed: {}", p.id(), e),
            }
        }
        all
    }
}
