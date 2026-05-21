//! MCP 配置检查 — 5 项（真扫 ~/.claude/.mcp.json / ~/.cursor/mcp.json 等）
use super::{home, path_exists, SimpleCheck};
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};
use std::fs;

/// MCP 配置候选路径（按存在顺序）
const MCP_CONFIG_PATHS: &[&str] = &[
    ".claude/.mcp.json",
    ".claude/mcp.json",
    ".cursor/mcp.json",
    ".codex/mcp.json",
    ".clawheart/mcp.json",
];

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        bcheck("MC-001", "未使用 .mcp.json 含 stdio servers（无法被代理）",
            Some("迁移到 HTTP/SSE mode 或接受 baseline-only 防护"), check_stdio_servers),
        bcheck("MC-002", "无 MCP server 使用 eval() 类工具",
            Some("禁用相关技能或更换实现"), check_eval_tools),
        bcheck("MC-003", "MCP server 描述未含可疑 prompt（同形字 / 隐藏指令）",
            Some("用 ClawHeart 重新扫描已装技能"), check_suspicious_prompts),
        bcheck("MC-004", "MCP 工具基线已冻结（防 rug-pull）",
            Some("启动 ClawHeart 后自动冻结当前会话；查看 mcp_tool_baselines 表"),
            check_baseline_frozen),
        bcheck("MC-005", "MCP server 进程不以 root/admin 运行",
            Some("以普通用户启动 Agent，移除 mcp.json 中 sudo / docker run -u 0 前缀"),
            check_root_runners),
    ]
}

fn bcheck(
    id: &'static str,
    desc: &'static str,
    remediation: Option<&'static str>,
    run_fn: fn() -> (CheckOutcome, Option<String>),
) -> Box<dyn AuditCheck> {
    Box::new(SimpleCheck {
        id,
        category: Category::McpConfig,
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

/// 收集所有存在的 mcp.json (rel_path, parsed_json)
fn collect_mcp_configs() -> Vec<(&'static str, serde_json::Value)> {
    MCP_CONFIG_PATHS
        .iter()
        .filter_map(|rel| read_json(rel).map(|v| (*rel, v)))
        .collect()
}

fn iter_servers<'a>(
    v: &'a serde_json::Value,
) -> impl Iterator<Item = (&'a str, &'a serde_json::Value)> {
    v.get("mcpServers")
        .and_then(|x| x.as_object())
        .into_iter()
        .flat_map(|m| m.iter().map(|(k, v)| (k.as_str(), v)))
}

// ──────────────────────────────────────────────────────────────────
// MC-001 stdio servers
// ──────────────────────────────────────────────────────────────────
fn check_stdio_servers() -> (CheckOutcome, Option<String>) {
    let mut found = Vec::new();
    for rel in MCP_CONFIG_PATHS {
        if path_exists(rel) {
            found.push(*rel);
        }
    }
    if found.is_empty() {
        return (CheckOutcome::Pass, Some("未发现 mcp.json".into()));
    }
    (
        CheckOutcome::Warn,
        Some(format!("发现 {}，需手动确认 transport 类型", found.join(", "))),
    )
}

// ──────────────────────────────────────────────────────────────────
// MC-002 eval-class tools
// ──────────────────────────────────────────────────────────────────
fn check_eval_tools() -> (CheckOutcome, Option<String>) {
    let configs = collect_mcp_configs();
    if configs.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现 mcp.json".into()));
    }

    // 危险 command / args 关键字（覆盖动态执行 / shell 转义场景）
    let red_flags = [
        "eval", "exec(", "execvp", "subprocess",
        "/bin/sh", "/bin/bash", "powershell.exe",
        "node -e", "python -c", "ruby -e", "perl -e", "deno eval",
    ];

    let mut hits: Vec<String> = Vec::new();
    let mut scanned_servers = 0;
    for (rel, json) in &configs {
        for (name, cfg) in iter_servers(json) {
            scanned_servers += 1;
            // 拼成可搜索文本：command + args + env
            let mut text = String::new();
            if let Some(c) = cfg.get("command").and_then(|x| x.as_str()) {
                text.push_str(c);
                text.push(' ');
            }
            if let Some(args) = cfg.get("args").and_then(|x| x.as_array()) {
                for a in args {
                    if let Some(s) = a.as_str() {
                        text.push_str(s);
                        text.push(' ');
                    }
                }
            }
            let lower = text.to_lowercase();
            for needle in &red_flags {
                if lower.contains(needle) {
                    hits.push(format!("{}::{} 含 \"{}\"", rel, name, needle));
                    break;
                }
            }
        }
    }
    if hits.is_empty() {
        return (
            CheckOutcome::Pass,
            Some(format!("已扫描 {} 个 MCP server，无 eval/shell 工具", scanned_servers)),
        );
    }
    (CheckOutcome::Fail, Some(hits.join("; ")))
}

// ──────────────────────────────────────────────────────────────────
// MC-003 suspicious prompts (homoglyph + injection needles)
// ──────────────────────────────────────────────────────────────────
fn check_suspicious_prompts() -> (CheckOutcome, Option<String>) {
    let configs = collect_mcp_configs();
    if configs.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现 mcp.json".into()));
    }

    // 西里尔字母同形字：a/e/o/p/c/x （视觉与 ASCII 几乎一致）
    let homoglyphs: &[char] = &['а', 'е', 'о', 'р', 'с', 'х'];
    let needles = [
        "ignore previous",
        "disregard prior",
        "you must always",
        "always use this tool first",
        "no confirmation needed",
        "ignore safety",
        "system:",
    ];

    let mut hits: Vec<String> = Vec::new();
    let mut scanned = 0;
    for (rel, json) in &configs {
        for (name, cfg) in iter_servers(json) {
            scanned += 1;
            // 收集所有 string 字段（description / instructions / tools[].description）
            let mut blobs: Vec<String> = Vec::new();
            collect_strings(cfg, &mut blobs);
            for blob in &blobs {
                let lower = blob.to_lowercase();
                for needle in &needles {
                    if lower.contains(needle) {
                        hits.push(format!("{}::{} 含可疑指令 \"{}\"", rel, name, needle));
                    }
                }
                if blob.chars().any(|c| homoglyphs.contains(&c)) {
                    hits.push(format!("{}::{} 含西里尔同形字", rel, name));
                }
            }
        }
    }
    if hits.is_empty() {
        return (
            CheckOutcome::Pass,
            Some(format!("已扫描 {} 个 MCP server 字段", scanned)),
        );
    }
    (CheckOutcome::Fail, Some(hits.join("; ")))
}

fn collect_strings(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(arr) => {
            for x in arr {
                collect_strings(x, out);
            }
        }
        serde_json::Value::Object(map) => {
            for (_k, x) in map {
                collect_strings(x, out);
            }
        }
        _ => {}
    }
}

// ──────────────────────────────────────────────────────────────────
// MC-004 baseline frozen
// ──────────────────────────────────────────────────────────────────
fn check_baseline_frozen() -> (CheckOutcome, Option<String>) {
    // 检查 ClawHeart DB 中 mcp_tool_baselines 是否有数据
    #[cfg(feature = "storage")]
    {
        let Some(db_path) = dirs::home_dir().map(|h| h.join(".clawheart-v2").join("clawheart.db"))
        else {
            return (CheckOutcome::Skipped, Some("无 home 目录".into()));
        };
        if !db_path.is_file() {
            return (
                CheckOutcome::Skipped,
                Some("ClawHeart DB 不存在 — 启动应用后会自动冻结".into()),
            );
        }
        let conn = match rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            Ok(c) => c,
            Err(_) => return (CheckOutcome::Skipped, Some("DB 不可读".into())),
        };
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mcp_tool_baselines", [], |row| row.get(0))
            .unwrap_or(0);
        if count > 0 {
            return (
                CheckOutcome::Pass,
                Some(format!("已冻结 {} 条 baseline", count)),
            );
        }
        return (
            CheckOutcome::Warn,
            Some("mcp_tool_baselines 表为空 — 启动 Agent 并发起请求后会自动冻结".into()),
        );
    }
    #[cfg(not(feature = "storage"))]
    {
        (CheckOutcome::Skipped, Some("storage feature 未启用".into()))
    }
}

// ──────────────────────────────────────────────────────────────────
// MC-005 root runners
// ──────────────────────────────────────────────────────────────────
fn check_root_runners() -> (CheckOutcome, Option<String>) {
    let configs = collect_mcp_configs();
    if configs.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现 mcp.json".into()));
    }

    let mut hits: Vec<String> = Vec::new();
    let mut scanned = 0;
    for (rel, json) in &configs {
        for (name, cfg) in iter_servers(json) {
            scanned += 1;
            let mut cmd = String::new();
            if let Some(c) = cfg.get("command").and_then(|x| x.as_str()) {
                cmd.push_str(c);
                cmd.push(' ');
            }
            if let Some(args) = cfg.get("args").and_then(|x| x.as_array()) {
                for a in args {
                    if let Some(s) = a.as_str() {
                        cmd.push_str(s);
                        cmd.push(' ');
                    }
                }
            }
            let lower = cmd.to_lowercase();
            // sudo / pkexec / doas / docker run -u 0 / runas
            let bad = [
                ("sudo ", "sudo"),
                ("pkexec ", "pkexec"),
                ("doas ", "doas"),
                ("runas ", "runas"),
                ("-u 0", "docker run -u 0"),
                ("--user=root", "--user=root"),
                ("--user root", "--user root"),
            ];
            for (needle, label) in &bad {
                if lower.contains(needle) {
                    hits.push(format!("{}::{} 使用 {}", rel, name, label));
                    break;
                }
            }
        }
    }
    if hits.is_empty() {
        return (
            CheckOutcome::Pass,
            Some(format!("已扫描 {} 个 server，无 root 提权命令", scanned)),
        );
    }
    (CheckOutcome::Fail, Some(hits.join("; ")))
}
