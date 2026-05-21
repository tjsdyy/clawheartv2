//! 熔断器 — 4 状态机（CC-Switch 借鉴）
//!
//! Closed → HalfOpen → Open → Closed
//! 加一个 Quarantine 状态用于"短期内反复失败的上游"长冷却。

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Closed,      // 正常
    Open,        // 熔断中，拒绝新请求
    HalfOpen,    // 试探：放一个请求过去
    Quarantine,  // 长冷却
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    open_duration: Duration,
    quarantine_threshold: u32,
    quarantine_duration: Duration,

    failure_count: AtomicU32,
    success_count: AtomicU32,
    consecutive_open_cycles: AtomicU32,
    next_attempt_at: AtomicU64, // unix seconds
    state: std::sync::Mutex<State>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            open_duration: Duration::from_secs(30),
            quarantine_threshold: 3,
            quarantine_duration: Duration::from_secs(300),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            consecutive_open_cycles: AtomicU32::new(0),
            next_attempt_at: AtomicU64::new(0),
            state: std::sync::Mutex::new(State::Closed),
        }
    }
}

impl CircuitBreaker {
    pub fn allow(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Closed => true,
            State::Open | State::Quarantine => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                if now >= self.next_attempt_at.load(Ordering::Relaxed) {
                    *state = State::HalfOpen;
                    true
                } else {
                    false
                }
            }
            State::HalfOpen => true,
        }
    }

    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::HalfOpen => {
                let n = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if n >= self.success_threshold {
                    *state = State::Closed;
                    self.success_count.store(0, Ordering::SeqCst);
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.consecutive_open_cycles.store(0, Ordering::SeqCst);
                }
            }
            State::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        match *state {
            State::Closed => {
                let n = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if n >= self.failure_threshold {
                    *state = State::Open;
                    self.next_attempt_at.store(now + self.open_duration.as_secs(), Ordering::Relaxed);
                    self.consecutive_open_cycles.fetch_add(1, Ordering::SeqCst);
                }
            }
            State::HalfOpen => {
                *state = State::Open;
                self.success_count.store(0, Ordering::SeqCst);
                let cycles = self.consecutive_open_cycles.fetch_add(1, Ordering::SeqCst) + 1;
                let cooldown = if cycles >= self.quarantine_threshold {
                    *state = State::Quarantine;
                    self.quarantine_duration
                } else {
                    self.open_duration
                };
                self.next_attempt_at.store(now + cooldown.as_secs(), Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn state(&self) -> State {
        *self.state.lock().unwrap()
    }
}
