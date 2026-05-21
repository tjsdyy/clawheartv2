//! 80 项审计的各类目实现

pub mod agent_behavior;
pub mod credentials;
pub mod file_permission;
pub mod mcp_config;
pub mod network;
pub mod sandbox;
pub mod skills;
pub mod windows;

use crate::security::scanner::{AuditCheck, CheckOutcome, CheckResult, Category};

/// 创建一个简单的检查实例（W17 会用 macro 减少样板代码）。
pub(crate) struct SimpleCheck {
    pub id: &'static str,
    pub category: Category,
    pub description: &'static str,
    pub remediation: Option<&'static str>,
    pub run_fn: fn() -> (CheckOutcome, Option<String>),
}

impl AuditCheck for SimpleCheck {
    fn id(&self) -> &'static str { self.id }
    fn category(&self) -> Category { self.category }
    fn description(&self) -> &'static str { self.description }
    fn run(&self) -> CheckResult {
        let (outcome, detail) = (self.run_fn)();
        CheckResult {
            id: self.id,
            category: self.category,
            outcome,
            description: self.description,
            detail,
            remediation: self.remediation,
        }
    }
}

pub(crate) fn home(rel: &str) -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(rel))
}

pub(crate) fn path_exists(rel: &str) -> bool {
    home(rel).is_some_and(|p| p.exists())
}
