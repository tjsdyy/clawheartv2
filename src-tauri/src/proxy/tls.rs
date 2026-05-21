//! CA 颁发 + per-host cert 缓存（W6 完整接入 rcgen）
//!
//! 设计：
//!   - 根 CA 一次生成（~/.clawheart-v2/ca/clawheart-ca.pem + clawheart-ca.key.enc）
//!   - per-host cert 用根 CA 即时签发 + 内存 LRU + 磁盘缓存（~/.clawheart-v2/ca/hosts/）
//!   - 私钥用 OS Keychain 加密（DPAPI on Windows）

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct CaStatus {
    pub exists: bool,
    pub trusted: bool,
    pub valid_until: Option<String>,
    pub fingerprint_sha256: Option<String>,
}

pub fn ca_dir() -> PathBuf {
    crate::state::data_dir().join("ca")
}

pub fn ca_cert_path() -> PathBuf { ca_dir().join("clawheart-ca.pem") }
pub fn ca_key_enc_path() -> PathBuf { ca_dir().join("clawheart-ca.key.enc") }
pub fn host_cache_dir() -> PathBuf { ca_dir().join("hosts") }

#[cfg(feature = "proxy_real")]
pub fn ensure_ca() -> Result<CaStatus, String> {
    if !ca_cert_path().exists() {
        generate_ca()?;
    }
    Ok(CaStatus {
        exists: true,
        trusted: false, // 真实状态需用 security_framework / certutil 查询
        valid_until: None,
        fingerprint_sha256: ca_cert_fingerprint().ok(),
    })
}

#[cfg(feature = "proxy_real")]
fn generate_ca() -> Result<(), String> {
    use rcgen::*;
    std::fs::create_dir_all(ca_dir()).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(host_cache_dir()).map_err(|e| e.to_string())?;

    let mut params = CertificateParams::new(vec!["ClawHeart Desktop CA".into()])
        .map_err(|e| e.to_string())?;
    params.distinguished_name.push(DnType::CommonName, "ClawHeart Desktop CA");
    params.distinguished_name.push(DnType::OrganizationName, "ClawHeart");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.not_before = rcgen::date_time_ymd(2026, 1, 1);
    params.not_after = rcgen::date_time_ymd(2036, 1, 1);

    let key = KeyPair::generate().map_err(|e| e.to_string())?;
    let cert = params.self_signed(&key).map_err(|e| e.to_string())?;

    std::fs::write(ca_cert_path(), cert.pem()).map_err(|e| e.to_string())?;
    // 私钥：W6 用 keyring 加密后写 .enc 文件；alpha 先明文写（仅用于测试）
    std::fs::write(ca_key_enc_path(), key.serialize_pem()).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "proxy_real")]
fn ca_cert_fingerprint() -> Result<String, String> {
    let pem = std::fs::read(ca_cert_path()).map_err(|e| e.to_string())?;
    Ok(crate::security::sha256::hex_string_bytes(&pem))
}

#[cfg(not(feature = "proxy_real"))]
pub fn ensure_ca() -> Result<CaStatus, String> {
    Ok(CaStatus {
        exists: false,
        trusted: false,
        valid_until: None,
        fingerprint_sha256: None,
    })
}
