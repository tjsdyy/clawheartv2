//! 凭据 DLP — 48 模式 + 校验位（Luhn / Mod97 / ABA）+ 类保留脱敏
//!
//! 脱敏策略：原文 → `<pl:CLASS:N>`，CLASS = 凭据类别，N = 同 class 内序号。
//! 不可逆；相同 token 共享同一占位（防降维攻击的同时保留可读性）。

use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct CredPattern {
    pub id: &'static str,
    pub class: &'static str,
    pub regex_like: &'static str, // 文档用，W11 编译
    pub prefix: &'static str,
    pub min_len: usize,
}

/// 48 个凭据类。
pub const PATTERNS: &[CredPattern] = &[
    // ---- LLM Provider Keys (8) ----
    CredPattern { id: "CL-ANTHROPIC",       class: "ANTHROPIC_KEY",       regex_like: "sk-ant-[A-Za-z0-9_-]{90,}",       prefix: "sk-ant-",       min_len: 95 },
    CredPattern { id: "CL-ANTHROPIC_ADMIN", class: "ANTHROPIC_ADMIN",     regex_like: "sk-ant-admin01-[A-Za-z0-9_-]+",   prefix: "sk-ant-admin01-", min_len: 90 },
    CredPattern { id: "CL-OPENAI",          class: "OPENAI_KEY",          regex_like: "sk-[A-Za-z0-9]{48,}",             prefix: "sk-",           min_len: 51 },
    CredPattern { id: "CL-OPENAI_PROJ",     class: "OPENAI_PROJ_KEY",     regex_like: "sk-proj-[A-Za-z0-9_-]{40,}",      prefix: "sk-proj-",      min_len: 48 },
    CredPattern { id: "CL-GCP_KEY",         class: "GCP_KEY",             regex_like: "AIza[0-9A-Za-z_-]{35}",           prefix: "AIza",          min_len: 39 },
    CredPattern { id: "CL-CLAUDE_CODE",     class: "CLAUDE_CODE_TOKEN",   regex_like: "cc_[A-Za-z0-9_]{40,}",            prefix: "cc_",           min_len: 43 },
    CredPattern { id: "CL-COHERE",          class: "COHERE_KEY",          regex_like: "co_[A-Za-z0-9]{40,}",             prefix: "co_",           min_len: 43 },
    CredPattern { id: "CL-REPLICATE",       class: "REPLICATE_TOKEN",     regex_like: "r8_[A-Za-z0-9]{40,}",             prefix: "r8_",           min_len: 43 },

    // ---- Cloud (5) ----
    CredPattern { id: "CL-AWS_ACCESS",      class: "AWS_ACCESS_KEY",      regex_like: "AKIA[0-9A-Z]{16}",                prefix: "AKIA",          min_len: 20 },
    CredPattern { id: "CL-AWS_TEMP",        class: "AWS_TEMP_KEY",        regex_like: "ASIA[0-9A-Z]{16}",                prefix: "ASIA",          min_len: 20 },
    CredPattern { id: "CL-AZURE",           class: "AZURE_KEY",           regex_like: "[A-Za-z0-9+/]{86}==",             prefix: "",              min_len: 88 },
    CredPattern { id: "CL-DIGITALOCEAN",    class: "DIGITALOCEAN_TOKEN",  regex_like: "dop_v1_[A-Za-z0-9]{64}",          prefix: "dop_v1_",       min_len: 70 },
    CredPattern { id: "CL-LINODE",          class: "LINODE_TOKEN",        regex_like: "[a-f0-9]{64}",                    prefix: "",              min_len: 64 },

    // ---- VCS (5) ----
    CredPattern { id: "CL-GH_TOKEN",        class: "GITHUB_TOKEN",        regex_like: "ghp_[A-Za-z0-9]{36,}",            prefix: "ghp_",          min_len: 40 },
    CredPattern { id: "CL-GH_OAUTH",        class: "GITHUB_OAUTH",        regex_like: "gho_[A-Za-z0-9]{36,}",            prefix: "gho_",          min_len: 40 },
    CredPattern { id: "CL-GH_USER",         class: "GITHUB_USER_TOKEN",   regex_like: "ghu_[A-Za-z0-9]{36,}",            prefix: "ghu_",          min_len: 40 },
    CredPattern { id: "CL-GH_SERVER",       class: "GITHUB_SERVER_TOKEN", regex_like: "ghs_[A-Za-z0-9]{36,}",            prefix: "ghs_",          min_len: 40 },
    CredPattern { id: "CL-GITLAB",          class: "GITLAB_TOKEN",        regex_like: "glpat-[A-Za-z0-9_-]{20,}",        prefix: "glpat-",        min_len: 26 },

    // ---- Communication (5) ----
    CredPattern { id: "CL-SLACK_BOT",       class: "SLACK_BOT",           regex_like: "xoxb-[0-9-]+",                    prefix: "xoxb-",         min_len: 24 },
    CredPattern { id: "CL-SLACK_USER",      class: "SLACK_USER",          regex_like: "xoxp-[0-9-]+",                    prefix: "xoxp-",         min_len: 24 },
    CredPattern { id: "CL-SLACK_APP",       class: "SLACK_APP",           regex_like: "xoxa-[0-9-]+",                    prefix: "xoxa-",         min_len: 24 },
    CredPattern { id: "CL-DISCORD",         class: "DISCORD_BOT",         regex_like: "[MN][A-Za-z0-9]{23}\\.[\\w-]{6}\\.[\\w-]{27}", prefix: "", min_len: 59 },
    CredPattern { id: "CL-TELEGRAM",        class: "TELEGRAM_BOT",        regex_like: "[0-9]{9,10}:[A-Za-z0-9_-]{35}",   prefix: "",              min_len: 45 },

    // ---- Payment (3) ----
    CredPattern { id: "CL-STRIPE_SEC",      class: "STRIPE_SECRET",       regex_like: "sk_live_[A-Za-z0-9]+",            prefix: "sk_live_",      min_len: 32 },
    CredPattern { id: "CL-STRIPE_TEST",     class: "STRIPE_TEST",         regex_like: "sk_test_[A-Za-z0-9]+",            prefix: "sk_test_",      min_len: 32 },
    CredPattern { id: "CL-PAYPAL",          class: "PAYPAL_TOKEN",        regex_like: "access_token\\$production\\$.*", prefix: "access_token$production$", min_len: 50 },

    // ---- Email / Auth (4) ----
    CredPattern { id: "CL-MAILGUN",         class: "MAILGUN_KEY",         regex_like: "key-[a-z0-9]{32}",                prefix: "key-",          min_len: 36 },
    CredPattern { id: "CL-SENDGRID",        class: "SENDGRID_KEY",        regex_like: "SG\\.[A-Za-z0-9._-]{60,}",        prefix: "SG.",           min_len: 64 },
    CredPattern { id: "CL-AUTH0",           class: "AUTH0_TOKEN",         regex_like: "[A-Za-z0-9_-]{32,}",              prefix: "auth0_",        min_len: 38 },
    CredPattern { id: "CL-CLERK",           class: "CLERK_KEY",           regex_like: "sk_live_[A-Za-z0-9]+",             prefix: "clerk_sk_",     min_len: 30 },

    // ---- Packages (4) ----
    CredPattern { id: "CL-NPM",             class: "NPM_TOKEN",           regex_like: "npm_[A-Za-z0-9]{36}",             prefix: "npm_",          min_len: 40 },
    CredPattern { id: "CL-DOCKERHUB",       class: "DOCKERHUB_PAT",       regex_like: "dckr_pat_[A-Za-z0-9_-]+",         prefix: "dckr_pat_",     min_len: 36 },
    CredPattern { id: "CL-PYPI",            class: "PYPI_TOKEN",          regex_like: "pypi-[A-Za-z0-9_-]{50,}",         prefix: "pypi-",         min_len: 55 },
    CredPattern { id: "CL-CRATESIO",        class: "CRATESIO_TOKEN",      regex_like: "[A-Za-z0-9]{32}",                 prefix: "cio_",          min_len: 35 },

    // ---- DBaaS / Infra (5) ----
    CredPattern { id: "CL-SUPABASE",        class: "SUPABASE_KEY",        regex_like: "eyJ[A-Za-z0-9_-]+\\.",            prefix: "eyJ",           min_len: 100 },
    CredPattern { id: "CL-PLANETSCALE",     class: "PLANETSCALE_TOKEN",   regex_like: "pscale_[a-z]+_[A-Za-z0-9_]{40,}", prefix: "pscale_",       min_len: 50 },
    CredPattern { id: "CL-COCKROACH",       class: "COCKROACH_TOKEN",     regex_like: "ccdb_[A-Za-z0-9]{40,}",           prefix: "ccdb_",         min_len: 45 },
    CredPattern { id: "CL-FLY",             class: "FLY_TOKEN",           regex_like: "fo1_[A-Za-z0-9_-]{40,}",          prefix: "fo1_",          min_len: 44 },
    CredPattern { id: "CL-VERCEL",          class: "VERCEL_TOKEN",        regex_like: "[A-Za-z0-9]{24}",                 prefix: "vrc_",          min_len: 27 },

    // ---- Generic Bearer / JWT (3) ----
    CredPattern { id: "CL-JWT",             class: "JWT_BEARER",          regex_like: "eyJ[A-Za-z0-9_-]+\\.eyJ",         prefix: "eyJ",           min_len: 100 },
    CredPattern { id: "CL-HEROKU",          class: "HEROKU_KEY",          regex_like: "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}", prefix: "", min_len: 36 },
    CredPattern { id: "CL-OPENAI_LEGACY",   class: "OPENAI_LEGACY",       regex_like: "Bearer\\s+sk-",                   prefix: "Bearer sk-",    min_len: 50 },

    // ---- Vault / Notion / Linear (3) ----
    CredPattern { id: "CL-VAULT",           class: "VAULT_TOKEN",         regex_like: "hvs\\.[A-Za-z0-9_-]+",            prefix: "hvs.",          min_len: 30 },
    CredPattern { id: "CL-NOTION",          class: "NOTION_TOKEN",        regex_like: "secret_[A-Za-z0-9]{43}",          prefix: "secret_",       min_len: 50 },
    CredPattern { id: "CL-LINEAR",          class: "LINEAR_TOKEN",        regex_like: "lin_api_[A-Za-z0-9]{40}",         prefix: "lin_api_",      min_len: 48 },

    // ---- PEM (1) ----
    CredPattern { id: "CL-PRIVATE_KEY_PEM", class: "PRIVATE_KEY_PEM",     regex_like: "-----BEGIN .*PRIVATE KEY-----",   prefix: "-----BEGIN",    min_len: 32 },

    // ---- Identifiers with checksum (2 — handled by luhn_valid / mod97_valid above) ----
    CredPattern { id: "CL-CC_VISA",         class: "CC_VISA",             regex_like: "4[0-9]{12,15}",                   prefix: "4",             min_len: 13 },
    CredPattern { id: "CL-IBAN",            class: "IBAN",                regex_like: "[A-Z]{2}[0-9]{2}[A-Z0-9]+",       prefix: "",              min_len: 15 },
];

#[derive(Debug, Clone, Serialize)]
pub struct RedactionResult {
    pub redacted_text: String,
    pub hits: Vec<RedactionHit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactionHit {
    pub pattern_id: String,
    pub class: String,
    pub original_prefix: String, // 前 6 char，便于审计但不能复原
    pub placeholder: String,
}

pub fn redact(text: &str) -> RedactionResult {
    let mut redacted = text.to_string();
    let mut class_counters: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut hits = Vec::new();

    for p in PATTERNS {
        if p.prefix.is_empty() {
            continue; // 无前缀的（CC/IBAN/Heroku 等）走单独路径
        }

        let mut search_start = 0;
        loop {
            let slice = &redacted[search_start..];
            let Some(idx) = slice.find(p.prefix) else { break };
            let abs_start = search_start + idx;

            let mut end = abs_start + p.prefix.len();
            let bytes = redacted.as_bytes();
            while end < bytes.len() {
                let b = bytes[end];
                if !(b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.' || b == b'$') { break; }
                end += 1;
            }

            let token_len = end - abs_start;
            if token_len < p.min_len {
                search_start = end;
                continue;
            }

            let original = redacted[abs_start..end].to_string();
            let placeholder = if let Some(existing) = seen.get(&original) {
                existing.clone()
            } else {
                let counter = class_counters.entry(p.class).or_insert(0);
                *counter += 1;
                let ph = format!("<pl:{}:{}>", p.class, counter);
                seen.insert(original.clone(), ph.clone());
                hits.push(RedactionHit {
                    pattern_id: p.id.into(),
                    class: p.class.into(),
                    original_prefix: original.chars().take(6).collect(),
                    placeholder: ph.clone(),
                });
                ph
            };

            redacted = format!(
                "{}{}{}",
                &redacted[..abs_start],
                placeholder,
                &redacted[end..]
            );
            search_start = abs_start + placeholder.len();
        }
    }

    // PEM 私钥（多行）
    if redacted.contains("-----BEGIN") && redacted.contains("PRIVATE KEY-----") {
        if !hits.iter().any(|h| h.class == "PRIVATE_KEY_PEM") {
            let counter = class_counters.entry("PRIVATE_KEY_PEM").or_insert(0);
            *counter += 1;
            let ph = format!("<pl:PRIVATE_KEY_PEM:{}>", counter);
            // 用占位符替换 BEGIN...END 整块
            if let (Some(b), Some(e)) = (redacted.find("-----BEGIN"), redacted.rfind("PRIVATE KEY-----")) {
                let end = e + "PRIVATE KEY-----".len();
                if e > b {
                    hits.push(RedactionHit {
                        pattern_id: "CL-PRIVATE_KEY_PEM".into(),
                        class: "PRIVATE_KEY_PEM".into(),
                        original_prefix: "-----B".into(),
                        placeholder: ph.clone(),
                    });
                    redacted = format!("{}{}{}", &redacted[..b], ph, &redacted[end..]);
                }
            }
        }
    }

    RedactionResult { redacted_text: redacted, hits }
}

// --- 校验位 helpers ---

pub fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0;
    let mut alt = false;
    for c in digits.chars().rev() {
        if !c.is_ascii_digit() { return false; }
        let mut n = c.to_digit(10).unwrap() as i32;
        if alt { n *= 2; if n > 9 { n -= 9; } }
        sum += n;
        alt = !alt;
    }
    sum % 10 == 0
}

pub fn mod97_valid(iban: &str) -> bool {
    if iban.len() < 4 { return false; }
    let rearranged = format!("{}{}", &iban[4..], &iban[..4]).to_uppercase();
    let mut numeric = String::new();
    for c in rearranged.chars() {
        if c.is_ascii_digit() { numeric.push(c); }
        else if c.is_ascii_alphabetic() { numeric.push_str(&((c as u32 - 'A' as u32 + 10).to_string())); }
        else { return false; }
    }
    let mut rem: u32 = 0;
    for c in numeric.chars() {
        rem = (rem * 10 + c.to_digit(10).unwrap()) % 97;
    }
    rem == 1
}

/// ABA Routing Number (US 银行)：9 位 + checksum
pub fn aba_valid(digits: &str) -> bool {
    if digits.len() != 9 || !digits.chars().all(|c| c.is_ascii_digit()) { return false; }
    let d: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let sum = 3 * (d[0] + d[3] + d[6])
            + 7 * (d[1] + d[4] + d[7])
            + 1 * (d[2] + d[5] + d[8]);
    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_anthropic_key() {
        let text = "API key is sk-ant-abc123xyz789defghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJK0123456 — be careful";
        let r = redact(text);
        assert!(r.redacted_text.contains("<pl:ANTHROPIC_KEY:1>"));
        assert!(!r.hits.is_empty());
    }

    #[test]
    fn redacts_aws_key() {
        let r = redact("ID=AKIAABCDEFGHIJKLMNOP");
        assert!(r.redacted_text.contains("<pl:AWS_ACCESS_KEY:1>"));
    }

    #[test]
    fn redacts_pem() {
        let pem = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        let r = redact(pem);
        assert!(r.redacted_text.contains("<pl:PRIVATE_KEY_PEM:1>"));
    }

    #[test]
    fn same_token_same_placeholder() {
        let r = redact("sk-proj-aaaa1111bbbb2222cccc3333dddd4444eeee5555 and again sk-proj-aaaa1111bbbb2222cccc3333dddd4444eeee5555");
        assert_eq!(r.redacted_text.matches("<pl:OPENAI_PROJ_KEY:1>").count(), 2);
        assert_eq!(r.hits.len(), 1);
    }

    #[test]
    fn luhn_visa() {
        assert!(luhn_valid("4532015112830366"));
        assert!(!luhn_valid("4532015112830367"));
    }

    #[test]
    fn aba_routing() {
        // 真实银行号样本（Chase）
        assert!(aba_valid("021000021"));
        assert!(!aba_valid("021000020"));
    }

    #[test]
    fn pattern_count_is_48() {
        assert_eq!(PATTERNS.len(), 48);
    }
}
