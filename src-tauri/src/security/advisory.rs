//! 安全公告订阅 — Ed25519 签名 feed
//!
//! 后台 worker 每 6h 拉取 feed.json + feed.json.sig，
//! Ed25519 验证（公钥 hardcoded + 漂移守护：CI 每次发布检查 3 处副本一致），
//! 命中本机已装技能/Agent → 触发 intercept_event。
//!
//! 失败关闭：签名校验失败 → 当次轮询作废，不发任何告警；UI 显示"订阅异常"。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub affected: Vec<AffectedSpec>,
    pub cvss_score: Option<f64>,
    pub action: Option<String>,
    pub published: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedSpec {
    pub kind: String,         // skill | agent | mcp_server
    pub slug: String,
    pub version_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub generated_at: String,
    pub advisories: Vec<Advisory>,
}

/// Hardcoded 公钥 (Ed25519, 32 bytes raw)。
/// 生产环境部署前生成一份私钥（minisign / age），将 base64 公钥替换进来。
/// 漂移守护：CI 每次发布检查本仓库 + feed server + monitor server 三处一致。
pub const FEED_PUBLIC_KEY_BASE64: &str = "REPLACE_WITH_ED25519_PUBLIC_KEY_32_BYTES_BASE64";

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("signature mismatch")]
    SignatureMismatch,
    #[error("malformed signature ({0})")]
    Malformed(&'static str),
    #[error("public key not configured")]
    NoKey,
    #[error("ed25519-dalek not enabled (build with `feed_verify` feature)")]
    DalekDisabled,
}

/// 验证 feed.json 签名。
///
/// W17 启用 `feed_verify` feature 后接入 `ed25519-dalek`；
/// alpha 阶段保留 surface 但返回 NotEnabled 让上层走"feed 异常"分支（失败关闭）。
#[cfg(feature = "feed_verify")]
pub fn verify_signature(feed_bytes: &[u8], sig_bytes: &[u8]) -> Result<(), VerifyError> {
    use ed25519_dalek::{Signature, VerifyingKey, Verifier};

    // 用同仓库的 base64 解码器（security::redact::try_base64_decode 私有；这里复用类似实现）
    let pk_bytes = decode_b64_32(FEED_PUBLIC_KEY_BASE64).ok_or(VerifyError::NoKey)?;
    let vk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| VerifyError::Malformed("pk"))?;

    if sig_bytes.len() != 64 { return Err(VerifyError::Malformed("sig len")); }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(sig_bytes);
    let sig = Signature::from_bytes(&sig_arr);

    vk.verify(feed_bytes, &sig).map_err(|_| VerifyError::SignatureMismatch)
}

#[cfg(not(feature = "feed_verify"))]
pub fn verify_signature(_feed_bytes: &[u8], _sig_bytes: &[u8]) -> Result<(), VerifyError> {
    Err(VerifyError::DalekDisabled)
}

#[cfg(feature = "feed_verify")]
fn decode_b64_32(s: &str) -> Option<[u8; 32]> {
    // 极简 base64 decode；与 normalizer 的实现保持一致
    let table: [u8; 256] = {
        let mut t = [0xFFu8; 256];
        for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            t[*c as usize] = i as u8;
        }
        t
    };
    let cleaned: Vec<u8> = s.bytes().filter(|b| *b != b'=' && table[*b as usize] != 0xFF).collect();
    let mut out = Vec::with_capacity(33);
    for chunk in cleaned.chunks(4) {
        let mut buf = [0u8; 4];
        for (i, b) in chunk.iter().enumerate() { buf[i] = table[*b as usize]; }
        let n = chunk.len();
        if n >= 2 { out.push((buf[0] << 2) | (buf[1] >> 4)); }
        if n >= 3 { out.push((buf[1] << 4) | (buf[2] >> 2)); }
        if n == 4 { out.push((buf[2] << 6) | buf[3]); }
    }
    if out.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&out);
        Some(arr)
    } else { None }
}

/// 公告匹配 — 命中本机已装技能/Agent。
pub fn match_against_local(
    feed: &Feed,
    installed_skills: &[(&str, &str)], // (slug, version)
    discovered_agents: &[(&str, &str)],
) -> Vec<String> {
    let mut matched = Vec::new();
    for adv in &feed.advisories {
        for spec in &adv.affected {
            match spec.kind.as_str() {
                "skill" => {
                    if installed_skills.iter().any(|(s, _v)| s == &spec.slug) {
                        matched.push(adv.id.clone());
                    }
                }
                "agent" => {
                    if discovered_agents.iter().any(|(p, _v)| p == &spec.slug) {
                        matched.push(adv.id.clone());
                    }
                }
                _ => {}
            }
        }
    }
    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_finds_installed_skill() {
        let feed = Feed {
            generated_at: "2026-05-17".into(),
            advisories: vec![Advisory {
                id: "CVE-2026-8305".into(),
                severity: "high".into(),
                title: "test".into(),
                description: "x".into(),
                affected: vec![AffectedSpec {
                    kind: "skill".into(), slug: "@some/postgres-mcp".into(),
                    version_range: "<=1.4.1".into(),
                }],
                cvss_score: None, action: None, published: "2026-05-15".into(), references: vec![],
            }],
        };
        let installed: &[(&str, &str)] = &[("@some/postgres-mcp", "1.4.0")];
        let agents: &[(&str, &str)] = &[];
        let m = match_against_local(&feed, installed, agents);
        assert_eq!(m, vec!["CVE-2026-8305"]);
    }
}
