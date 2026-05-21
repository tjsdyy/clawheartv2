//! 凭据泄露检查 — 8 项（扫本机已知位置 + .bash_history 等）
use super::{home, SimpleCheck};
use crate::security::redact;
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};
use std::fs;

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        bcheck("CL-001", "~/.bash_history 不含明文凭据",
            Some("清理或 cleanup 历史：history -c && rm ~/.bash_history"),
            || scan_file(".bash_history")),
        bcheck("CL-002", "~/.zsh_history 不含明文凭据",
            Some("清理或 cleanup 历史：history -c && rm ~/.zsh_history"),
            || scan_file(".zsh_history")),
        bcheck("CL-003", "~/.psql_history 不含明文凭据",
            Some("清理或迁移到 .pgpass"), || scan_file(".psql_history")),
        bcheck("CL-004", "环境变量未明文持久化凭据（~/.bashrc / .zshrc / .profile）",
            Some("迁移到 .envrc + direnv 或 OS Keychain"),
            || {
                let mut leaks = Vec::new();
                for f in &[".bashrc", ".zshrc", ".profile"] {
                    if let Some(p) = home(f) {
                        if let Ok(s) = fs::read_to_string(&p) {
                            let r = redact::redact(&s);
                            if !r.hits.is_empty() {
                                leaks.push(format!("{}: {}", f, r.hits.len()));
                            }
                        }
                    }
                }
                if leaks.is_empty() { (CheckOutcome::Pass, None) }
                else { (CheckOutcome::Fail, Some(leaks.join(", "))) }
            }),
        bcheck("CL-005", "VS Code settings.json 不含凭据",
            Some("迁移到 OS Keychain"),
            || {
                #[cfg(target_os = "macos")] let p = home("Library/Application Support/Code/User/settings.json");
                #[cfg(target_os = "linux")] let p = home(".config/Code/User/settings.json");
                #[cfg(target_os = "windows")] let p = home("AppData/Roaming/Code/User/settings.json");
                match p.and_then(|pp| fs::read_to_string(&pp).ok()) {
                    Some(s) => {
                        let r = redact::redact(&s);
                        if r.hits.is_empty() { (CheckOutcome::Pass, None) }
                        else { (CheckOutcome::Fail, Some(format!("{} 个凭据", r.hits.len()))) }
                    }
                    None => (CheckOutcome::Skipped, Some("VS Code 未安装".into())),
                }
            }),
        bcheck("CL-006", "Cursor settings.json 不含凭据",
            Some("迁移到 OS Keychain"),
            || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        bcheck("CL-007", "Agent CLAUDE.md / MEMORY.md 不含凭据",
            Some("脱敏或删除相关条目"),
            || scan_file(".claude/CLAUDE.md")),
        bcheck("CL-008", "git config 不含 url-with-token 形式凭据",
            Some("用 credential helper 或 SSH"),
            || {
                let p = home(".gitconfig").and_then(|p| fs::read_to_string(&p).ok()).unwrap_or_default();
                if p.contains("@github.com") && p.contains("ghp_") {
                    (CheckOutcome::Fail, Some("git config 含 ghp_ token URL".into()))
                } else {
                    (CheckOutcome::Pass, None)
                }
            }),
    ]
}

fn scan_file(rel: &str) -> (CheckOutcome, Option<String>) {
    let Some(p) = home(rel) else { return (CheckOutcome::Skipped, None); };
    if !p.exists() { return (CheckOutcome::Skipped, Some(format!("{} 不存在", rel))); }
    let Ok(content) = fs::read_to_string(&p) else {
        return (CheckOutcome::Skipped, Some(format!("{} 不可读", rel)));
    };
    let r = redact::redact(&content);
    if r.hits.is_empty() {
        (CheckOutcome::Pass, None)
    } else {
        let classes: Vec<&str> = r.hits.iter().map(|h| h.class.as_str()).collect();
        (CheckOutcome::Fail, Some(format!("{} 个凭据: {}", r.hits.len(), classes.join(","))))
    }
}

fn bcheck(id: &'static str, desc: &'static str, remediation: Option<&'static str>,
         run_fn: fn() -> (CheckOutcome, Option<String>)) -> Box<dyn AuditCheck> {
    Box::new(SimpleCheck { id, category: Category::CredentialLeak, description: desc, remediation, run_fn })
}
