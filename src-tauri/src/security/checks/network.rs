//! 网络暴露检查 — 9 项
use super::SimpleCheck;
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        b("NE-001", "ClawHeart 代理端口仅监听 127.0.0.1",
          Some("不要 0.0.0.0 绑定（避免局域网暴露）"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-002", "Agent 出站只走 ClawHeart 代理（base URL 一致）",
          Some("export ANTHROPIC_BASE_URL=http://127.0.0.1:19111"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-003", "Agent 不访问明显的恶意域名（黑名单）",
          Some("ClawHeart 默认拦截已知 C2 域名"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-004", "Agent 不通过未声明的 webhook 上报数据",
          Some("阻止 webhook.site / requestbin / ngrok"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-005", "DNS 查询不使用未授权的 DoH/DoT",
          Some("配置 systemd-resolved 或 1.1.1.1 显式"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-006", "ClawHeart CA 已正确信任",
          Some("检查 Keychain / cert store"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-007", "无 reverse SSH tunnel 异常进程",
          Some("ps aux | grep ssh.*-R"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-008", "防火墙规则未被 AI 工具修改",
          Some("查看 iptables / pf / Windows Firewall 历史"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("NE-009", "无可疑监听端口（≥1024）",
          Some("netstat -tlnp 审查"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
    ]
}

fn b(id: &'static str, desc: &'static str, remediation: Option<&'static str>,
     run_fn: fn() -> (CheckOutcome, Option<String>)) -> Box<dyn AuditCheck> {
    Box::new(SimpleCheck { id, category: Category::NetworkExposure, description: desc, remediation, run_fn })
}
