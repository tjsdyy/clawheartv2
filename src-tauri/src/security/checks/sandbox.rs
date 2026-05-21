//! 沙箱与 Docker 检查 — 11 项
use super::SimpleCheck;
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        b("SB-001", "Agent 容器未挂载宿主 /",
          Some("docker run -v / 是危险操作"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-002", "Agent 容器未挂载 /var/run/docker.sock",
          Some("禁止 docker.sock 挂载（防容器逃逸）"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-003", "Agent 容器未挂载宿主 $HOME",
          Some("仅挂载需要的子目录"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-004", "Agent 容器未使用 --privileged",
          Some("移除 --privileged"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-005", "Agent 容器未使用 host network",
          Some("移除 --network=host"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-006", "Agent 容器以非 root 用户运行",
          Some("Dockerfile 使用 USER 指令"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-007", "Agent 容器有 cap_drop=ALL（默认能力最小化）",
          Some("docker run --cap-drop=ALL --cap-add=...只需的能力"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-008", "Agent 容器有内存限制（防资源耗尽）",
          Some("docker run -m 4g"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-009", "Agent 容器有 read-only root fs",
          Some("docker run --read-only + 必要的 tmpfs"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-010", "macOS Agent 未禁用 SIP",
          Some("csrutil status 应为 enabled"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
        b("SB-011", "Linux 启用 AppArmor / SELinux",
          Some("apparmor_status / sestatus 检查"), || (CheckOutcome::Skipped, Some("未实现 · 待 W21 接入".into()))),
    ]
}

fn b(id: &'static str, desc: &'static str, remediation: Option<&'static str>,
     run_fn: fn() -> (CheckOutcome, Option<String>)) -> Box<dyn AuditCheck> {
    Box::new(SimpleCheck { id, category: Category::SandboxDocker, description: desc, remediation, run_fn })
}
