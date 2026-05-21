//! 跨请求分片外泄检测 — Pipelock 心智
//!
//! 攻击者把一段 secret 分批通过多个请求泄露到同一域名 → 单请求 DLP 不触发。
//! 应对：按 (agent, target_host) 维护字节预算 + N 分钟滑动窗口；
//! 累积超过阈值 → 触发 cross_request_exfiltration 事件。

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy)]
pub struct BudgetConfig {
    pub window: Duration,
    pub max_bytes_per_window: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_secs(300), // 5 min
            max_bytes_per_window: 10 * 1024,   // 10 KB
        }
    }
}

pub struct CrossRequestBudget {
    cfg: BudgetConfig,
    /// (agent_id, host) → Vec<(at, bytes)>
    log: HashMap<(String, String), Vec<(SystemTime, usize)>>,
}

impl CrossRequestBudget {
    pub fn new(cfg: BudgetConfig) -> Self {
        Self { cfg, log: HashMap::new() }
    }

    /// 记录一次出站；返回当前窗口内累积字节。
    /// 若超过阈值 → 返回 Some(累积)，调用方触发拦截 + 事件。
    pub fn record_outbound(&mut self, agent: &str, host: &str, bytes: usize) -> Option<usize> {
        let key = (agent.to_string(), host.to_string());
        let now = SystemTime::now();
        let entries = self.log.entry(key).or_default();
        entries.push((now, bytes));

        // GC 过期条目
        entries.retain(|(at, _)| now.duration_since(*at).unwrap_or_default() < self.cfg.window);

        let total: usize = entries.iter().map(|(_, b)| b).sum();
        if total > self.cfg.max_bytes_per_window {
            Some(total)
        } else {
            None
        }
    }
}
