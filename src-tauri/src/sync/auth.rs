//! 云端认证 — token 走 OS Keychain
//!
//! 命令：login / logout / refresh_token / current_user
//! Keychain key 命名：clawheart.token / clawheart.refresh / clawheart.user

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: String,
    pub email: String,
    pub access_token_redacted: String, // 前 8 char + ...
    pub expires_at: String,
}

#[cfg(feature = "storage")]
pub mod backed {
    // W3 + keyring crate 接入
}
