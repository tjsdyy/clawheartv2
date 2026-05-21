//! 代理 HTTP/HTTPS 服务 — hudsucker 0.22 MITM 接入点
//!
//! W5 真 spike：用 hudsucker 0.22 完整集成：
//!   1. CA 自动生成 + 加载（proxy/ca_manager.rs）
//!   2. RcgenAuthority 作为 MITM 根
//!   3. ClawHeartHandler 同时实现 HttpHandler + WebSocketHandler
//!   4. Proxy::builder() 完整构造链 + start().await
//!
//! 启动方式：
//!   `clawheart` 主进程内 spawn（lib.rs 设置 proxy_real feature 时启用），
//!   或单测时手动调用 ProxyServer::new(...).start()。

use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub bind_host: String,
    pub started_at: Option<String>,
}

pub struct ProxyServer {
    pub port: u16,
    pub bind_host: String,
    #[allow(dead_code)]
    pub kill_switch: Arc<crate::security::kill_switch::KillSwitch>,
    #[allow(dead_code)]
    pub app: crate::state::AppState,
}

impl ProxyServer {
    pub fn new(
        port: u16,
        kill_switch: Arc<crate::security::kill_switch::KillSwitch>,
        app: crate::state::AppState,
    ) -> Self {
        Self {
            port,
            bind_host: "127.0.0.1".into(),
            kill_switch,
            app,
        }
    }

    /// 启动监听。
    /// - `proxy_real` feature 启用时：真实 hudsucker MITM 代理
    /// - 否则：返回错误，保留 stub 行为
    #[cfg(feature = "proxy_real")]
    pub async fn start(&self) -> Result<(), String> {
        use hudsucker::{certificate_authority::RcgenAuthority, Proxy};
        use std::net::SocketAddr;
        use std::sync::atomic::Ordering;

        let running_flag = self.app.proxy_server_running.clone();
        let port_flag = self.app.proxy_server_port.clone();

        // 1. 加载 / 生成 CA
        let ca_dir = super::ca_manager::default_ca_dir();
        let loaded = super::ca_manager::load_or_init(&ca_dir)?;
        if loaded.generated_now {
            tracing::warn!(
                ca_path = %ca_dir.display(),
                fingerprint = %loaded.fingerprint_sha256_hex,
                "ClawHeart CA 首次生成，用户需将其加入系统受信任根证书才能解密 HTTPS"
            );
        }

        // 2. 构造 hudsucker 的 Authority（缓存大小 1000 = 同时为 1000 个不同 host 签发 leaf cert）
        let authority = RcgenAuthority::new(loaded.key_pair, loaded.ca_cert, 1000);

        // 3. 解析监听地址
        let addr: SocketAddr = format!("{}:{}", self.bind_host, self.port)
            .parse()
            .map_err(|e: std::net::AddrParseError| format!("invalid bind addr: {}", e))?;

        // 4. 构造 handler（同时用于 HTTP 与 WebSocket）
        let handler = super::handler::ClawHeartHandler::new(
            self.kill_switch.clone(),
            self.app.clone(),
        );

        // 5. ProxyBuilder 完整链（hudsucker 0.22 状态机顺序：
        //    with_addr → with_rustls_client() 无参 → with_ca → handlers → build()）
        let proxy = Proxy::builder()
            .with_addr(addr)
            .with_rustls_client()
            .with_ca(authority)
            .with_http_handler(handler.clone())
            .with_websocket_handler(handler)
            .build();

        tracing::info!(addr = %addr, "hudsucker proxy starting");
        port_flag.store(self.port, Ordering::Release);
        running_flag.store(true, Ordering::Release);

        // 6. 启动（阻塞当前 task 到 proxy 退出）
        let result = proxy
            .start()
            .await
            .map_err(|e| format!("hudsucker proxy crashed: {}", e));

        // 退出时清标志
        running_flag.store(false, Ordering::Release);
        result
    }

    #[cfg(not(feature = "proxy_real"))]
    pub async fn start(&self) -> Result<(), String> {
        Err("proxy_real feature disabled".into())
    }
}
