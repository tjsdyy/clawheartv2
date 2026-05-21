//! Windows 专属检查 — 2 项
use super::SimpleCheck;
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        b("WIN-001", "Windows Defender 启用 + 实时保护",
          Some("Get-MpComputerStatus | Select RealTimeProtectionEnabled"),
          || {
              #[cfg(not(target_os = "windows"))]
              { (CheckOutcome::Skipped, Some("非 Windows 平台".into())) }
              #[cfg(target_os = "windows")]
              { (CheckOutcome::Skipped, Some("未实现 · 待 W21 接 wmi 真实查询".into())) }
          }),
        b("WIN-002", "PowerShell 执行策略不为 Bypass",
          Some("Set-ExecutionPolicy RemoteSigned -Scope CurrentUser"),
          || {
              #[cfg(not(target_os = "windows"))]
              { (CheckOutcome::Skipped, Some("非 Windows 平台".into())) }
              #[cfg(target_os = "windows")]
              { (CheckOutcome::Skipped, Some("未实现 · 待 W21 接 PowerShell".into())) }
          }),
    ]
}

fn b(id: &'static str, desc: &'static str, remediation: Option<&'static str>,
     run_fn: fn() -> (CheckOutcome, Option<String>)) -> Box<dyn AuditCheck> {
    Box::new(SimpleCheck { id, category: Category::WindowsSpecific, description: desc, remediation, run_fn })
}
