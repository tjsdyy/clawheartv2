//! 文件权限检查 — 10 项
use super::{path_exists, SimpleCheck};
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        b(SimpleCheck {
            id: "FP-001", category: Category::FilePermission,
            description: "~/.claude 目录权限不宽松（≤755）",
            remediation: Some("chmod 755 ~/.claude"),
            run_fn: || check_dir_perm(".claude", 0o755),
        }),
        b(SimpleCheck {
            id: "FP-002", category: Category::FilePermission,
            description: "~/.codex 目录权限不宽松",
            remediation: Some("chmod 755 ~/.codex"),
            run_fn: || check_dir_perm(".codex", 0o755),
        }),
        b(SimpleCheck {
            id: "FP-003", category: Category::FilePermission,
            description: ".env 文件权限为 600（仅当前用户读写）",
            remediation: Some("chmod 600 .env"),
            run_fn: check_env_perm,
        }),
        b(SimpleCheck {
            id: "FP-004", category: Category::FilePermission,
            description: "~/.ssh/id_rsa 权限为 600",
            remediation: Some("chmod 600 ~/.ssh/id_rsa"),
            run_fn: || check_file_perm(".ssh/id_rsa", 0o600),
        }),
        b(SimpleCheck {
            id: "FP-005", category: Category::FilePermission,
            description: "~/.aws/credentials 权限为 600",
            remediation: Some("chmod 600 ~/.aws/credentials"),
            run_fn: || check_file_perm(".aws/credentials", 0o600),
        }),
        b(SimpleCheck {
            id: "FP-006", category: Category::FilePermission,
            description: "Agent 配置文件不在世界可读位置（/tmp 等）",
            remediation: Some("移到 $HOME 下"),
            run_fn: || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into())),
        }),
        b(SimpleCheck {
            id: "FP-007", category: Category::FilePermission,
            description: "关键配置文件启用 immutable 标志（chattr +i / chflags uchg）",
            remediation: Some("chattr +i ~/.claude/CLAUDE.md (Linux) 或 chflags uchg (macOS)"),
            run_fn: || (CheckOutcome::Warn, Some("可选，启用后修改需先去标志".into())),
        }),
        b(SimpleCheck {
            id: "FP-008", category: Category::FilePermission,
            description: "ClawHeart 数据目录 ~/.clawheart-v2 权限正确",
            remediation: Some("chmod 700 ~/.clawheart-v2"),
            run_fn: || check_dir_perm(".clawheart-v2", 0o700),
        }),
        b(SimpleCheck {
            id: "FP-009", category: Category::FilePermission,
            description: "Keychain 加密的私钥文件无明文备份",
            remediation: Some("移除 ~/.clawheart-v2/ca/*.key（应只有 *.enc）"),
            run_fn: || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into())),
        }),
        b(SimpleCheck {
            id: "FP-010", category: Category::FilePermission,
            description: "应用程序 binary 已签名",
            remediation: Some("codesign --verify $(which clawheart) (macOS)"),
            run_fn: || (CheckOutcome::Skipped, Some("在 dev 模式下跳过".into())),
        }),
    ]
}

fn b(c: SimpleCheck) -> Box<dyn AuditCheck> { Box::new(c) }

#[cfg(unix)]
fn check_dir_perm(rel: &str, max_mode: u32) -> (CheckOutcome, Option<String>) {
    use std::os::unix::fs::MetadataExt;
    let Some(p) = super::home(rel) else { return (CheckOutcome::Skipped, None) };
    if !p.exists() { return (CheckOutcome::Skipped, Some(format!("{} 不存在", rel))); }
    match std::fs::metadata(&p) {
        Ok(m) => {
            let mode = m.mode() & 0o777;
            if mode <= max_mode {
                (CheckOutcome::Pass, Some(format!("{:o}", mode)))
            } else {
                (CheckOutcome::Fail, Some(format!("权限 {:o}, 期望 ≤ {:o}", mode, max_mode)))
            }
        }
        Err(e) => (CheckOutcome::Fail, Some(e.to_string())),
    }
}

#[cfg(windows)]
fn check_dir_perm(_rel: &str, _max_mode: u32) -> (CheckOutcome, Option<String>) {
    (CheckOutcome::Skipped, Some("Windows 用 ACL 检查（W17）".into()))
}

#[cfg(unix)]
fn check_file_perm(rel: &str, expected: u32) -> (CheckOutcome, Option<String>) {
    use std::os::unix::fs::MetadataExt;
    let Some(p) = super::home(rel) else { return (CheckOutcome::Skipped, None) };
    if !p.exists() { return (CheckOutcome::Skipped, Some(format!("{} 不存在", rel))); }
    let mode = std::fs::metadata(&p).map(|m| m.mode() & 0o777).unwrap_or(0);
    if mode == expected {
        (CheckOutcome::Pass, Some(format!("{:o}", mode)))
    } else {
        (CheckOutcome::Fail, Some(format!("权限 {:o}, 期望 {:o}", mode, expected)))
    }
}

#[cfg(windows)]
fn check_file_perm(_rel: &str, _expected: u32) -> (CheckOutcome, Option<String>) {
    (CheckOutcome::Skipped, Some("Windows 用 ACL 检查（W17）".into()))
}

fn check_env_perm() -> (CheckOutcome, Option<String>) {
    if path_exists(".env") {
        check_file_perm(".env", 0o600)
    } else {
        (CheckOutcome::Skipped, Some(".env 不存在".into()))
    }
}
