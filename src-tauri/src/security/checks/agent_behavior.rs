//! Agent 行为检查 — 8 项（真扫 ~/.claude / ~/.codex / ~/.cursor 配置）
//!
//! 设计原则：
//! - 找不到目标 Agent → Skipped（"未安装"）
//! - 找到但字段缺失 → Pass（默认配置不危险）
//! - 找到危险配置 → Fail / Warn 并附带证据
use super::{home, SimpleCheck};
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};
use std::fs;

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        b("AB-001", "Claude Code 未启用 auto-approve（所有 shell 命令需确认）",
          Some("移除 settings.json 中的 autoApprove / dangerouslySkipPermissions 字段"),
          check_claude_auto_approve),
        b("AB-002", "Codex 未启用 dangerous-skip-permissions",
          Some("不要传 --dangerously-skip-permissions；移除 config.toml 中 approval_policy = \"never\""),
          check_codex_dangerous_skip),
        b("AB-003", "Cursor MCP server 不含未授权的 stdio runners",
          Some("审查 mcp.json 中 mcpServers 字段，移除未识别条目"),
          check_cursor_mcp_servers),
        b("AB-004", "Agent 配置文件未启用 telemetry / analytics",
          Some("settings.json: telemetry.enabled = false（或对应 Agent 关闭 metrics）"),
          check_telemetry),
        b("AB-005", "Agent 未连接到非官方代理（除 ClawHeart 之外）",
          Some("ANTHROPIC_BASE_URL / OPENAI_BASE_URL 只指向 127.0.0.1:19111/19112 或官方端点"),
          check_base_url_unofficial),
        b("AB-006", "Agent CLAUDE.md / AGENTS.md 不含被注入的 prompt 指令",
          Some("用 ClawHeart 注入扫描器检查 markdown 中的可疑指令"),
          check_memory_injection),
        b("AB-007", "Agent session 历史目录未被多用户共享访问",
          Some("chmod 700 ~/.claude/projects（或对应路径）"),
          check_session_dir_perm),
        b("AB-008", "Agent 日志未含未脱敏凭据",
          Some("清理日志或启用 ClawHeart 流式 DLP"),
          check_log_credentials),
    ]
}

fn b(
    id: &'static str,
    desc: &'static str,
    remediation: Option<&'static str>,
    run_fn: fn() -> (CheckOutcome, Option<String>),
) -> Box<dyn AuditCheck> {
    Box::new(SimpleCheck {
        id,
        category: Category::AgentBehavior,
        description: desc,
        remediation,
        run_fn,
    })
}

// ──────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────

fn read_json(rel: &str) -> Option<serde_json::Value> {
    let p = home(rel)?;
    if !p.is_file() {
        return None;
    }
    let text = fs::read_to_string(&p).ok()?;
    serde_json::from_str(&text).ok()
}

fn read_text(rel: &str) -> Option<String> {
    let p = home(rel)?;
    if !p.is_file() {
        return None;
    }
    fs::read_to_string(&p).ok()
}

// ──────────────────────────────────────────────────────────────────
// AB-001 Claude Code auto-approve
// ──────────────────────────────────────────────────────────────────
fn check_claude_auto_approve() -> (CheckOutcome, Option<String>) {
    let Some(v) = read_json(".claude/settings.json") else {
        return (CheckOutcome::Skipped, Some("~/.claude/settings.json 不存在".into()));
    };
    let mut hits: Vec<String> = Vec::new();
    if let Some(b) = v.get("autoApprove").and_then(|x| x.as_bool()) {
        if b {
            hits.push("autoApprove = true".into());
        }
    }
    if let Some(b) = v.get("dangerouslySkipPermissions").and_then(|x| x.as_bool()) {
        if b {
            hits.push("dangerouslySkipPermissions = true".into());
        }
    }
    // Anthropic 还有 permissions.allowAll / autoApproveTools 等字段
    if let Some(perms) = v.get("permissions") {
        if perms.get("allowAll").and_then(|x| x.as_bool()).unwrap_or(false) {
            hits.push("permissions.allowAll = true".into());
        }
    }
    if !hits.is_empty() {
        return (
            CheckOutcome::Fail,
            Some(format!("检测到危险字段：{}", hits.join(", "))),
        );
    }
    (CheckOutcome::Pass, Some("未启用 auto-approve / skip-permissions".into()))
}

// ──────────────────────────────────────────────────────────────────
// AB-002 Codex dangerous-skip-permissions
// ──────────────────────────────────────────────────────────────────
fn check_codex_dangerous_skip() -> (CheckOutcome, Option<String>) {
    let Some(text) = read_text(".codex/config.toml") else {
        return (CheckOutcome::Skipped, Some("~/.codex/config.toml 不存在".into()));
    };
    let lower = text.to_lowercase();
    let mut hits: Vec<&str> = Vec::new();
    // Codex 用 approval_policy = "never" / disable_safe_mode = true 等
    if lower.contains("approval_policy") && lower.contains("\"never\"") {
        hits.push("approval_policy = \"never\"");
    }
    if lower.contains("disable_safe_mode") && lower.contains("true") {
        hits.push("disable_safe_mode = true");
    }
    if lower.contains("dangerously_skip_permissions") && lower.contains("true") {
        hits.push("dangerously_skip_permissions = true");
    }
    if !hits.is_empty() {
        return (
            CheckOutcome::Fail,
            Some(format!("检测到：{}", hits.join(", "))),
        );
    }
    (CheckOutcome::Pass, Some("未启用 skip-permissions".into()))
}

// ──────────────────────────────────────────────────────────────────
// AB-003 Cursor MCP servers
// ──────────────────────────────────────────────────────────────────
fn check_cursor_mcp_servers() -> (CheckOutcome, Option<String>) {
    // 优先查 ~/.cursor/mcp.json，回退 ~/.cursor/settings.json
    let v = read_json(".cursor/mcp.json").or_else(|| read_json(".cursor/settings.json"));
    let Some(v) = v else {
        return (CheckOutcome::Skipped, Some("~/.cursor/mcp.json 不存在".into()));
    };
    let servers = v
        .get("mcpServers")
        .and_then(|x| x.as_object());
    let Some(servers) = servers else {
        return (CheckOutcome::Pass, Some("未配置 mcpServers".into()));
    };

    // 名单：列出所有 stdio runner（含 command 字段且 type ≠ http/sse）
    let mut stdio_servers: Vec<String> = Vec::new();
    for (name, cfg) in servers {
        let kind = cfg.get("type").and_then(|x| x.as_str()).unwrap_or("stdio");
        if kind != "http" && kind != "sse" && cfg.get("command").is_some() {
            stdio_servers.push(name.clone());
        }
    }
    if stdio_servers.is_empty() {
        return (CheckOutcome::Pass, Some(format!("共 {} 个非 stdio MCP server", servers.len())));
    }
    // 有 stdio runner → Warn（用户需要审查）
    (
        CheckOutcome::Warn,
        Some(format!(
            "{} 个 stdio MCP server 需手动审查：{}",
            stdio_servers.len(),
            stdio_servers.join(", "),
        )),
    )
}

// ──────────────────────────────────────────────────────────────────
// AB-004 Telemetry
// ──────────────────────────────────────────────────────────────────
fn check_telemetry() -> (CheckOutcome, Option<String>) {
    let mut enabled: Vec<&str> = Vec::new();
    let mut checked = 0;

    // Claude
    if let Some(v) = read_json(".claude/settings.json") {
        checked += 1;
        let on = v
            .get("telemetry")
            .and_then(|t| t.get("enabled"))
            .and_then(|x| x.as_bool())
            .unwrap_or(true); // Anthropic 默认开启
        if on {
            enabled.push("Claude");
        }
    }
    // Cursor
    if let Some(v) = read_json(".cursor/settings.json") {
        checked += 1;
        // Cursor: telemetry.telemetryLevel ≠ "off"
        let level = v
            .get("telemetry.telemetryLevel")
            .and_then(|x| x.as_str())
            .unwrap_or_else(|| {
                v.get("telemetry")
                    .and_then(|t| t.get("telemetryLevel"))
                    .and_then(|x| x.as_str())
                    .unwrap_or("all")
            });
        if level != "off" && level != "crash" {
            enabled.push("Cursor");
        }
    }

    if checked == 0 {
        return (CheckOutcome::Skipped, Some("未发现 Claude / Cursor 配置".into()));
    }
    if enabled.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已检查 {} 个 Agent，遥测均关闭", checked)));
    }
    (
        CheckOutcome::Warn,
        Some(format!("默认开启遥测：{}", enabled.join(", "))),
    )
}

// ──────────────────────────────────────────────────────────────────
// AB-005 Unofficial base URL
// ──────────────────────────────────────────────────────────────────
fn check_base_url_unofficial() -> (CheckOutcome, Option<String>) {
    let mut hits: Vec<String> = Vec::new();
    let mut checked = 0;

    let official_anthropic = ["api.anthropic.com"];
    let official_openai = ["api.openai.com", "openai.azure.com"];
    let allowed_local = ["127.0.0.1", "localhost", "::1"];

    let is_allowed = |url: &str| -> bool {
        let u = url.to_lowercase();
        official_anthropic.iter().any(|h| u.contains(h))
            || official_openai.iter().any(|h| u.contains(h))
            || allowed_local.iter().any(|h| u.contains(h))
    };

    // Claude settings.json: env.ANTHROPIC_BASE_URL
    if let Some(v) = read_json(".claude/settings.json") {
        checked += 1;
        if let Some(url) = v
            .get("env")
            .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
            .and_then(|x| x.as_str())
        {
            if !is_allowed(url) {
                hits.push(format!("Claude.ANTHROPIC_BASE_URL → {}", url));
            }
        }
    }
    // Codex config.toml: base_url 字段（粗扫文本）
    if let Some(text) = read_text(".codex/config.toml") {
        checked += 1;
        for line in text.lines() {
            let l = line.trim();
            if l.starts_with("base_url") && l.contains('=') {
                if let Some(rest) = l.split_once('=').map(|(_, r)| r) {
                    let url = rest.trim().trim_matches('"');
                    if !url.is_empty() && !is_allowed(url) {
                        hits.push(format!("Codex.base_url → {}", url));
                    }
                }
            }
        }
    }
    // Cursor settings.json: cursor.openaiBaseUrl
    if let Some(v) = read_json(".cursor/settings.json") {
        checked += 1;
        let url_opt = v
            .get("cursor.openaiBaseUrl")
            .and_then(|x| x.as_str())
            .or_else(|| {
                v.get("cursor")
                    .and_then(|c| c.get("openaiBaseUrl"))
                    .and_then(|x| x.as_str())
            });
        if let Some(url) = url_opt {
            if !is_allowed(url) {
                hits.push(format!("Cursor.openaiBaseUrl → {}", url));
            }
        }
    }

    if checked == 0 {
        return (CheckOutcome::Skipped, Some("未发现 Agent 配置".into()));
    }
    if hits.is_empty() {
        return (
            CheckOutcome::Pass,
            Some(format!("已检查 {} 个 Agent，无非官方代理", checked)),
        );
    }
    (
        CheckOutcome::Warn,
        Some(format!("非官方代理：{}", hits.join("; "))),
    )
}

// ──────────────────────────────────────────────────────────────────
// AB-006 Memory file prompt injection
// ──────────────────────────────────────────────────────────────────
fn check_memory_injection() -> (CheckOutcome, Option<String>) {
    let candidates = [
        ".claude/CLAUDE.md",
        ".claude/MEMORY.md",
        ".codex/AGENTS.md",
        ".cursor/AGENTS.md",
    ];
    let mut scanned: Vec<&str> = Vec::new();
    let mut hits: Vec<String> = Vec::new();

    // 复用 injection 模块的 needles（部分关键词）
    let red_flags = [
        "ignore previous instructions",
        "ignore all previous",
        "disregard prior",
        "you are now",
        "system: you must",
        "tell me your prompt",
        "your system prompt",
        "exfiltrate",
    ];

    for rel in &candidates {
        if let Some(text) = read_text(rel) {
            scanned.push(rel);
            let lower = text.to_lowercase();
            for needle in &red_flags {
                if lower.contains(needle) {
                    hits.push(format!("{} 含 \"{}\"", rel, needle));
                }
            }
        }
    }

    if scanned.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现 CLAUDE.md / MEMORY.md / AGENTS.md".into()));
    }
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个 memory 文件", scanned.len())));
    }
    (
        CheckOutcome::Fail,
        Some(format!("可疑 prompt 指令：{}", hits.join("; "))),
    )
}

// ──────────────────────────────────────────────────────────────────
// AB-007 Session dir permissions
// ──────────────────────────────────────────────────────────────────
#[cfg(unix)]
fn check_session_dir_perm() -> (CheckOutcome, Option<String>) {
    use std::os::unix::fs::MetadataExt;
    let candidates = [
        ".claude/projects",
        ".claude/sessions",
        ".codex/sessions",
        ".cursor/sessions",
    ];
    let mut checked = 0;
    let mut bad: Vec<String> = Vec::new();
    for rel in &candidates {
        let Some(p) = home(rel) else { continue };
        if !p.exists() {
            continue;
        }
        checked += 1;
        if let Ok(meta) = fs::metadata(&p) {
            let mode = meta.mode() & 0o777;
            // group/other 可读或可写 → 不安全
            if mode & 0o077 != 0 {
                bad.push(format!("{} = {:o}", rel, mode));
            }
        }
    }
    if checked == 0 {
        return (CheckOutcome::Skipped, Some("未发现 session 目录".into()));
    }
    if bad.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已检查 {} 个 session 目录，权限均 ≤ 700", checked)));
    }
    (CheckOutcome::Fail, Some(format!("权限过宽：{}", bad.join(", "))))
}

#[cfg(windows)]
fn check_session_dir_perm() -> (CheckOutcome, Option<String>) {
    (CheckOutcome::Skipped, Some("Windows 用 ACL 检查（W21）".into()))
}

// ──────────────────────────────────────────────────────────────────
// AB-008 Log credentials
// ──────────────────────────────────────────────────────────────────
fn check_log_credentials() -> (CheckOutcome, Option<String>) {
    use crate::security::redact;

    let candidates = [
        ".claude/logs/latest.log",
        ".claude/logs/claude.log",
        ".codex/logs/codex.log",
        ".cursor/logs/cursor.log",
    ];
    let mut scanned: Vec<&str> = Vec::new();
    let mut hits: Vec<String> = Vec::new();
    for rel in &candidates {
        if let Some(text) = read_text(rel) {
            scanned.push(rel);
            // 大文件只扫前 256KB
            let snippet = if text.len() > 256 * 1024 {
                &text[..256 * 1024]
            } else {
                &text[..]
            };
            let r = redact::redact(snippet);
            if !r.hits.is_empty() {
                hits.push(format!("{}: {} 个凭据", rel, r.hits.len()));
            }
        }
    }
    if scanned.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现 Agent 日志".into()));
    }
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个日志，未发现凭据", scanned.len())));
    }
    (CheckOutcome::Fail, Some(hits.join("; ")))
}
