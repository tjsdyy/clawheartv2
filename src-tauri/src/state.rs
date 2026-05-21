// 全局共享状态。
//
// 设计：所有字段均为 cheap-clone 句柄（Arc / AtomicXxx），AppState 本体 derive Clone，
// 这样既可 .manage(state.clone()) 给 Tauri，也可 spawn 子任务（如 fetch_server）持有
// 同一份状态。

use crate::security::kill_switch::KillSwitch;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU16};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct UiState {
    pub theme: String,
    pub language: String,
    pub agents_discovered: u32,
    pub onboarded: bool,
}

#[derive(Clone)]
pub struct AppState {
    /// 应用 UI 偏好（W3 起持久化到 settings 表）。
    pub ui: Arc<Mutex<UiState>>,
    /// 4 源 OR Kill Switch（lib.rs 启动时构造）。
    pub kill_switch: Arc<KillSwitch>,
    /// 数据库连接（启用 storage feature 时）。
    #[cfg(feature = "storage")]
    pub db: Option<crate::storage::conn::SharedConnection>,
    /// fetch_server 运行时状态（W12 后用于 backend_ready 联动）
    pub fetch_server_running: Arc<AtomicBool>,
    pub fetch_server_port: Arc<AtomicU16>,
    /// hudsucker ProxyServer 运行时状态（W5 spike 后用于 tier2 backend_ready）
    pub proxy_server_running: Arc<AtomicBool>,
    pub proxy_server_port: Arc<AtomicU16>,
}

impl AppState {
    pub fn new() -> Self {
        let sentinel_path = data_dir().join("STOP");
        Self {
            ui: Arc::new(Mutex::new(UiState {
                theme: "paper".into(),
                language: "zh".into(),
                agents_discovered: 0,
                onboarded: false,
            })),
            kill_switch: Arc::new(KillSwitch::new(sentinel_path)),
            #[cfg(feature = "storage")]
            db: None,
            fetch_server_running: Arc::new(AtomicBool::new(false)),
            fetch_server_port: Arc::new(AtomicU16::new(19112)),
            proxy_server_running: Arc::new(AtomicBool::new(false)),
            proxy_server_port: Arc::new(AtomicU16::new(19111)),
        }
    }

    #[cfg(feature = "storage")]
    pub fn with_db(mut self, db: crate::storage::conn::SharedConnection) -> Self {
        self.db = Some(db);
        self
    }
}

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".clawheart-v2")
}
