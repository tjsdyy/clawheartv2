//! IPC: 认证（login/logout/refresh）

use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginArgs {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResult {
    pub user_id: String,
    pub email: String,
    pub expires_at: String,
}

#[tauri::command]
pub fn login(args: LoginArgs) -> AppResult<AuthResult> {
    // W3 接入 reqwest + keyring
    Ok(AuthResult {
        user_id: "demo-user".into(),
        email: args.email,
        expires_at: "2099-01-01T00:00:00Z".into(),
    })
}

#[tauri::command]
pub fn logout() -> AppResult<()> {
    Ok(())
}

#[tauri::command]
pub fn refresh_token() -> AppResult<bool> {
    Ok(false) // not yet authenticated
}
