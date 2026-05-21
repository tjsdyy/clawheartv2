//! IPC: 代理服务管理
use crate::error::AppResult;
use serde::Serialize;

#[derive(Serialize)]
pub struct ProxyControlResult {
    pub running: bool,
    pub port: u16,
}

#[tauri::command]
pub fn proxy_pause() -> AppResult<ProxyControlResult> {
    Ok(ProxyControlResult { running: false, port: 19111 })
}

#[tauri::command]
pub fn proxy_resume() -> AppResult<ProxyControlResult> {
    Ok(ProxyControlResult { running: true, port: 19111 })
}

#[tauri::command]
pub fn proxy_get_ca_cert() -> AppResult<String> {
    Ok("~/.clawheart/ca/clawheart-ca.pem".into())
}

#[tauri::command]
pub fn proxy_install_ca() -> AppResult<bool> {
    // W6: 调用 security add-trusted-cert / certutil / nssdb
    Ok(false)
}
