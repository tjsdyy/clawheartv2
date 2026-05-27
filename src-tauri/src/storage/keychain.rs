//! OS Keychain 凭据存储（keyring crate 抽象层）
//!
//! 命名约定：
//!   - clawheart.token        — access token
//!   - clawheart.refresh      — refresh token
//!   - clawheart.user_id      — current user id
//!   - clawheart.ca_key       — CA private key (PEM encoded)
//!   - clawheart.provider.<id> — provider profile API key
//!
//! 双层存储策略：
//!   1. 优先用 OS Keychain（生产环境最安全）
//!   2. Keychain store 成功后做 read-back 验证；若 macOS Tauri dev 模式
//!      因签名/权限问题导致 store 成功但读不出，**自动降级到文件存储**
//!      （`~/.clawheart-v2/credentials/<key>`，chmod 0600，hex 编码）
//!   3. fetch 同样走 keychain → 文件 fallback 链
//!
//! 这避免了 dev 模式"无错误但 credential_set 永远为 false"的假阴性。

#[cfg(feature = "storage")]
pub use backed::*;

#[cfg(feature = "storage")]
mod backed {
    use keyring::Entry;
    use std::path::PathBuf;

    const SERVICE: &str = "live.clawheart.desktop";

    fn fallback_dir() -> PathBuf {
        let home = dirs::home_dir()
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".clawheart-v2/credentials")
    }

    fn fallback_path(key: &str) -> PathBuf {
        let safe = key.replace(['/', '\\', ':'], "_");
        fallback_dir().join(safe)
    }

    fn write_fallback(key: &str, secret: &str) -> Result<(), String> {
        let dir = fallback_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("create credentials dir failed: {}", e))?;
        let path = fallback_path(key);
        std::fs::write(&path, hex_encode(secret))
            .map_err(|e| format!("write credential file failed: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &path,
                std::fs::Permissions::from_mode(0o600),
            );
        }
        Ok(())
    }

    pub fn store(key: &str, secret: &str) -> Result<(), String> {
        // 优先 keychain
        let kc_ok = (|| -> Result<bool, String> {
            let entry = Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
            entry.set_password(secret).map_err(|e| e.to_string())?;
            // read-back 验证：macOS dev 模式可能 store ok 但 fetch 失败
            match entry.get_password() {
                Ok(s) if s == secret => Ok(true),
                _ => Ok(false),
            }
        })()
        .unwrap_or(false);

        if kc_ok {
            // Dev builds are often unsigned, which can make OS keychain access
            // flaky across process restarts. Keep a local shadow fallback there
            // so credential_set and proxy routing remain stable while developing.
            #[cfg(debug_assertions)]
            {
                write_fallback(key, secret)?;
            }
            #[cfg(not(debug_assertions))]
            {
                // keychain 完全可用，清理可能残留的 fallback 文件
                let _ = std::fs::remove_file(fallback_path(key));
            }
            return Ok(());
        }

        // Fallback: 文件存储（chmod 0600 + hex 编码）
        tracing::warn!(
            "keychain unavailable for key={}, fallback to ~/.clawheart-v2/credentials/",
            key
        );
        write_fallback(key, secret)
    }

    pub fn fetch(key: &str) -> Result<Option<String>, String> {
        // 优先 keychain
        if let Ok(entry) = Entry::new(SERVICE, key) {
            match entry.get_password() {
                Ok(s) => return Ok(Some(s)),
                Err(keyring::Error::NoEntry) => {}
                Err(_) => {} // 静默 fallback
            }
        }
        // Fallback: 文件
        let path = fallback_path(key);
        match std::fs::read_to_string(&path) {
            Ok(encoded) => hex_decode(encoded.trim())
                .map(Some)
                .map_err(|e| format!("decode credential file: {}", e)),
            Err(_) => Ok(None),
        }
    }

    pub fn delete(key: &str) -> Result<(), String> {
        // 同时清 keychain + 文件
        if let Ok(entry) = Entry::new(SERVICE, key) {
            let _ = entry.delete_credential();
        }
        let _ = std::fs::remove_file(fallback_path(key));
        Ok(())
    }

    // ──────────────────────────────────────────────────────────────────
    // Hex 编码（不引入新 crate；非加密，仅混淆 + chmod 0600 保护）
    // ──────────────────────────────────────────────────────────────────
    fn hex_encode(s: &str) -> String {
        s.bytes().map(|b| format!("{:02x}", b)).collect()
    }

    fn hex_decode(s: &str) -> Result<String, String> {
        if s.len() % 2 != 0 {
            return Err("invalid hex length".into());
        }
        let mut bytes = Vec::with_capacity(s.len() / 2);
        for chunk in s.as_bytes().chunks(2) {
            let hex_str = std::str::from_utf8(chunk).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(hex_str, 16).map_err(|e| e.to_string())?;
            bytes.push(b);
        }
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn hex_roundtrip() {
            let s = "sk-test-abc-123";
            let enc = hex_encode(s);
            let dec = hex_decode(&enc).unwrap();
            assert_eq!(s, dec);
        }
    }
}

#[cfg(not(feature = "storage"))]
pub fn store(_key: &str, _secret: &str) -> Result<(), String> {
    Err("storage feature disabled".into())
}
#[cfg(not(feature = "storage"))]
pub fn fetch(_key: &str) -> Result<Option<String>, String> { Ok(None) }
#[cfg(not(feature = "storage"))]
pub fn delete(_key: &str) -> Result<(), String> { Ok(()) }
