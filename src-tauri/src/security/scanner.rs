//! 离线 80 项安全审计 — 收编自 OpenClaw Audit
//!
//! 8 类目 × N 项检查；每项独立实现，可单测、可单跑。
//! 触发点：「扫描」工具 UI 多选 → start_scan_run → 并发跑所有勾选项。

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    FilePermission,
    McpConfig,
    CredentialLeak,
    AgentBehavior,
    SkillSupplyChain,
    SandboxDocker,
    NetworkExposure,
    WindowsSpecific,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::FilePermission   => "文件权限",
            Category::McpConfig        => "MCP 配置",
            Category::CredentialLeak   => "凭据泄露",
            Category::AgentBehavior    => "Agent 行为",
            Category::SkillSupplyChain => "技能供应链",
            Category::SandboxDocker    => "沙箱与 Docker",
            Category::NetworkExposure  => "网络暴露",
            Category::WindowsSpecific  => "Windows 专属",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    Pass,
    Fail,
    Warn,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub id: &'static str,
    pub category: Category,
    pub outcome: CheckOutcome,
    pub description: &'static str,
    pub detail: Option<String>,
    pub remediation: Option<&'static str>,
}

pub trait AuditCheck: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> Category;
    fn description(&self) -> &'static str;
    fn run(&self) -> CheckResult;
}

/// 返回所有内置检查（80 项）。
pub fn all_checks() -> Vec<Box<dyn AuditCheck>> {
    use crate::security::checks;
    let mut v: Vec<Box<dyn AuditCheck>> = Vec::with_capacity(80);
    v.extend(checks::file_permission::checks());
    v.extend(checks::mcp_config::checks());
    v.extend(checks::credentials::checks());
    v.extend(checks::agent_behavior::checks());
    v.extend(checks::skills::checks());
    v.extend(checks::sandbox::checks());
    v.extend(checks::network::checks());
    v.extend(checks::windows::checks());
    v
}

pub fn count_by_category() -> [(Category, usize); 8] {
    [
        (Category::FilePermission, 10),
        (Category::McpConfig, 5),
        (Category::CredentialLeak, 8),
        (Category::AgentBehavior, 8),
        (Category::SkillSupplyChain, 12),
        (Category::SandboxDocker, 11),
        (Category::NetworkExposure, 9),
        (Category::WindowsSpecific, 2),
    ]
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanRun {
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total_checks: u32,
    pub passed: u32,
    pub failed: u32,
    pub warned: u32,
    pub skipped: u32,
    pub results: Vec<CheckResult>,
}

/// 执行选中的检查。`selected_keys` 可为空（=全部），或为：
/// - check id（"FP-001"）
/// - category 名（"FilePermission"）
/// 任意一种匹配即跑该 check。
pub fn run_scan(selected_keys: &[&str]) -> ScanRun {
    run_scan_with(selected_keys, |_, _, _| {})
}

/// 带进度回调的扫描。回调收到 (index, total, result)，可用于 emit Tauri event。
pub fn run_scan_with<F>(selected_keys: &[&str], mut on_check: F) -> ScanRun
where
    F: FnMut(u32, u32, &CheckResult),
{
    let started = now_iso();
    let all = all_checks();

    // 预先过滤命中列表，便于回调里给出准确的 total
    let matched: Vec<&Box<dyn AuditCheck>> = all
        .iter()
        .filter(|check| {
            if selected_keys.is_empty() {
                return true;
            }
            let cat_name = format!("{:?}", check.category());
            selected_keys
                .iter()
                .any(|k| k.eq_ignore_ascii_case(&cat_name) || k.eq_ignore_ascii_case(check.id()))
        })
        .collect();

    let total = matched.len() as u32;
    let mut results = Vec::with_capacity(matched.len());
    let mut passed = 0;
    let mut failed = 0;
    let mut warned = 0;
    let mut skipped = 0;

    for (i, check) in matched.iter().enumerate() {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check.run()))
            .unwrap_or_else(|_| CheckResult {
                id: check.id(),
                category: check.category(),
                outcome: CheckOutcome::Fail,
                description: check.description(),
                detail: Some("check panicked — failed closed".into()),
                remediation: None,
            });
        match r.outcome {
            CheckOutcome::Pass => passed += 1,
            CheckOutcome::Fail => failed += 1,
            CheckOutcome::Warn => warned += 1,
            CheckOutcome::Skipped => skipped += 1,
        }
        on_check((i + 1) as u32, total, &r);
        results.push(r);
    }

    ScanRun {
        started_at: started,
        completed_at: Some(now_iso()),
        total_checks: results.len() as u32,
        passed,
        failed,
        warned,
        skipped,
        results,
    }
}

/// `YYYY-MM-DD HH:MM:SS` UTC（与 SQLite `datetime('now')` 同格式）
fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_iso_utc(secs)
}

fn format_iso_utc(epoch: u64) -> String {
    // Civil-from-days，Howard Hinnant 算法（无外部依赖）
    let days = (epoch / 86_400) as i64;
    let sod = epoch % 86_400;
    let hh = sod / 3600;
    let mm = (sod / 60) % 60;
    let ss = sod % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146097)
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 400)
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 366)
    let mp = (5 * doy + 2) / 153; // [0, 11)
    let d = (doy - (153 * mp + 2) / 5 + 1) as u64; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u64; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hh, mm, ss
    )
}

#[cfg(test)]
mod time_tests {
    use super::format_iso_utc;

    #[test]
    fn formats_epoch_zero() {
        assert_eq!(format_iso_utc(0), "1970-01-01 00:00:00");
    }

    #[test]
    fn formats_known_date() {
        // 2026-05-19 11:00:00 UTC = 1779188400
        assert_eq!(format_iso_utc(1_779_188_400), "2026-05-19 11:00:00");
    }
}
