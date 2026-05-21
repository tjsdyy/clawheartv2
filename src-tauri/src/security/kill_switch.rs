//! Kill Switch — 4 源 OR（任一激活即全拒）。
//!
//! 借鉴 Pipelock。每个请求入口处 [`KillSwitch::snapshot`] 一次（atomic snapshot）防 TOCTOU。
//!
//! 4 源：
//! 1. config_kill：DB settings.kill_switch=1（持久化）
//! 2. api_kill：用户在 UI 点击紧急停止（内存）
//! 3. signal_kill：SIGUSR1（仅 Unix）
//! 4. sentinel_path：~/.clawheart/STOP 文件存在 OR 不可读 = 激活（失败关闭）

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct KillSwitch {
    pub config_kill: AtomicBool,
    pub api_kill: AtomicBool,
    pub signal_kill: AtomicBool,
    pub sentinel_path: PathBuf,
}

impl KillSwitch {
    pub fn new(sentinel_path: PathBuf) -> Self {
        Self {
            config_kill: AtomicBool::new(false),
            api_kill: AtomicBool::new(false),
            signal_kill: AtomicBool::new(false),
            sentinel_path,
        }
    }

    /// 每个请求入口处调用一次。
    pub fn snapshot(&self) -> bool {
        if self.config_kill.load(Ordering::Acquire) { return true; }
        if self.api_kill.load(Ordering::Acquire)    { return true; }
        if self.signal_kill.load(Ordering::Acquire) { return true; }

        // 哨兵：存在 = 激活；不可读 = 激活（失败关闭）
        match std::fs::metadata(&self.sentinel_path) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(_) => true, // permission denied 等 → 激活
        }
    }

    /// 用户从 UI 点击紧急停止。
    pub fn activate_api(&self) {
        self.api_kill.store(true, Ordering::Release);
    }

    /// 用户手动恢复（UI / DB / 信号都得清，但不动哨兵文件）。
    pub fn reset(&self) {
        self.config_kill.store(false, Ordering::Release);
        self.api_kill.store(false, Ordering::Release);
        self.signal_kill.store(false, Ordering::Release);
    }
}

/// Unix SIGUSR1 注册（在 main.rs 启动时调用一次）。Windows 上 no-op。
#[cfg(unix)]
pub fn install_signal_handler(ks: std::sync::Arc<KillSwitch>) {
    // alpha: 实际信号注册留到 W11；这里仅占位
    let _ = ks;
}

#[cfg(not(unix))]
pub fn install_signal_handler(_: std::sync::Arc<KillSwitch>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn nothing_active_means_not_killed() {
        let ks = KillSwitch::new(PathBuf::from("/tmp/clawheart-test-nonexistent-stop"));
        assert!(!ks.snapshot());
    }

    #[test]
    fn api_activation_kills() {
        let ks = KillSwitch::new(PathBuf::from("/tmp/clawheart-test-nonexistent-stop"));
        ks.activate_api();
        assert!(ks.snapshot());
    }

    #[test]
    fn sentinel_file_kills() {
        let path = std::env::temp_dir().join("clawheart-ks-test-STOP");
        let _ = std::fs::File::create(&path).unwrap().write_all(b"");
        let ks = KillSwitch::new(path.clone());
        assert!(ks.snapshot());
        let _ = std::fs::remove_file(&path);
    }
}
