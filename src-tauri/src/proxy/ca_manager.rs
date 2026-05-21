//! CA 证书管理（hudsucker MITM 用）
//!
//! W5：首次启动自动生成自签 CA，持久化到 `~/.clawheart-v2/ca/`：
//!   clawheart-ca.pem      — CA 证书（用户需安装到系统受信任根存储）
//!   clawheart-ca.key      — CA 私钥（chmod 600；W6 转 Keychain）
//!
//! 后续启动：从 PEM 加载到 rcgen 类型，构造 RcgenAuthority。

#![cfg(feature = "proxy_real")]

use std::fs;
use std::path::{Path, PathBuf};

use hudsucker::rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair,
};

const CA_PEM_FILENAME: &str = "clawheart-ca.pem";
const CA_KEY_FILENAME: &str = "clawheart-ca.key";

const CA_COMMON_NAME: &str = "ClawHeart Local CA";
const CA_ORGANIZATION: &str = "ClawHeart Desktop";

pub fn default_ca_dir() -> PathBuf {
    crate::state::data_dir().join("ca")
}

pub struct LoadedCa {
    pub key_pair: KeyPair,
    pub ca_cert: hudsucker::rcgen::Certificate,
    pub fingerprint_sha256_hex: String,
    pub generated_now: bool,
}

/// 加载 CA；若不存在则自动生成并持久化。
pub fn load_or_init(ca_dir: &Path) -> Result<LoadedCa, String> {
    fs::create_dir_all(ca_dir)
        .map_err(|e| format!("create ca dir {:?} failed: {}", ca_dir, e))?;

    let pem_path = ca_dir.join(CA_PEM_FILENAME);
    let key_path = ca_dir.join(CA_KEY_FILENAME);

    if pem_path.exists() && key_path.exists() {
        let cert_pem = fs::read_to_string(&pem_path)
            .map_err(|e| format!("read {}: {}", pem_path.display(), e))?;
        let key_pem = fs::read_to_string(&key_path)
            .map_err(|e| format!("read {}: {}", key_path.display(), e))?;
        let (key_pair, ca_cert) = parse_ca_from_pem(&key_pem, &cert_pem)?;
        let fp = sha256_hex(cert_pem.as_bytes());
        tracing::info!(
            ca_path = %pem_path.display(),
            fingerprint = %fp,
            "ClawHeart CA loaded from disk"
        );
        return Ok(LoadedCa {
            key_pair,
            ca_cert,
            fingerprint_sha256_hex: fp,
            generated_now: false,
        });
    }

    // 首次启动：生成新 CA
    let (key_pem, cert_pem) = generate_new_ca_pem()?;
    fs::write(&pem_path, &cert_pem)
        .map_err(|e| format!("write {}: {}", pem_path.display(), e))?;
    fs::write(&key_path, &key_pem)
        .map_err(|e| format!("write {}: {}", key_path.display(), e))?;

    // Unix: chmod 600 for key file
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&key_path)
            .map_err(|e| format!("stat key: {}", e))?
            .permissions();
        perm.set_mode(0o600);
        let _ = fs::set_permissions(&key_path, perm);
    }

    let (key_pair, ca_cert) = parse_ca_from_pem(&key_pem, &cert_pem)?;
    let fp = sha256_hex(cert_pem.as_bytes());

    tracing::info!(
        ca_path = %pem_path.display(),
        fingerprint = %fp,
        "ClawHeart CA generated and persisted"
    );

    Ok(LoadedCa {
        key_pair,
        ca_cert,
        fingerprint_sha256_hex: fp,
        generated_now: true,
    })
}

fn parse_ca_from_pem(
    key_pem: &str,
    cert_pem: &str,
) -> Result<(KeyPair, hudsucker::rcgen::Certificate), String> {
    let key_pair = KeyPair::from_pem(key_pem)
        .map_err(|e| format!("parse CA private key: {}", e))?;
    let params = CertificateParams::from_ca_cert_pem(cert_pem)
        .map_err(|e| format!("parse CA certificate: {}", e))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| format!("self_sign CA: {}", e))?;
    Ok((key_pair, cert))
}

fn generate_new_ca_pem() -> Result<(String, String), String> {
    let key_pair = KeyPair::generate()
        .map_err(|e| format!("generate keypair: {}", e))?;

    let mut params = CertificateParams::new(vec![])
        .map_err(|e| format!("new params: {}", e))?;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, CA_COMMON_NAME);
    dn.push(DnType::OrganizationName, CA_ORGANIZATION);
    params.distinguished_name = dn;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    // 默认有效期由 rcgen 决定（通常 10 年）。
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| format!("self_sign new CA: {}", e))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    Ok((key_pem, cert_pem))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let d = crate::security::sha256::digest(bytes);
    crate::security::sha256::hex(&d)
}
