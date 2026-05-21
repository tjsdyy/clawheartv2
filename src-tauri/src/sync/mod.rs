//! 云端同步 — 后台 worker（tokio interval）
//!
//! 入口：`SyncManager::start_periodic()` 在 lib.rs setup 时启动。
//! W18 起接入真实 endpoint；现在仅骨架。

pub mod auth;
pub mod danger;
pub mod intercept;
pub mod skills;
pub mod usage;

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default, Serialize)]
pub struct SyncStatus {
    pub running: bool,
    pub last_sync_unix: u64,
    pub last_error: Option<String>,
}

pub struct SyncManager {
    running: AtomicBool,
}

impl Default for SyncManager {
    fn default() -> Self { Self::new() }
}

impl SyncManager {
    pub fn new() -> Self {
        Self { running: AtomicBool::new(false) }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// 启动周期性同步任务（每 5 分钟）。
    /// W18 实现真实 HTTP 调用；现在仅占位。
    pub fn start_periodic(&self) {
        self.running.store(true, Ordering::Release);
        // tokio::spawn 留到 W18
    }
}
