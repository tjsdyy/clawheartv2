//! 技能供应链检查 — 12 项（真接 SkillGuard 规则引擎）
//!
//! 数据源：
//!   - `crate::skills::discover::discover_all()` 通配扫描 ~/.<agent>/skills/
//!   - `crate::security::skill_scanner::scan(&bundle)` 跑 SkillGuard 规则
//!
//! 各 audit 项基于扫描结果 (ScanReport) 推导：
//!   - hard_triggers ⊆ {"SK-001" eval(input), "SK-002" socket/subprocess,
//!                       "SK-005" 同形字, "SK-008" base64-stage}
//!   - findings 是加权 (SK-101 描述可疑 / SK-102 hardcoded key / SK-103 网络)
//!
//! 注：scan_skills 模块内部规则 id 与本审计层 id 同前缀但不同空间，需注意区分。
use super::SimpleCheck;
use crate::security::scanner::{AuditCheck, Category, CheckOutcome};
use crate::security::skill_scanner::{self, Context, ScanReport, SkillBundle};
use crate::skills::discover::{discover_all, DiscoveredSkill};
use std::fs;
use std::path::Path;

pub fn checks() -> Vec<Box<dyn AuditCheck>> {
    vec![
        b("SK-001", "已装技能均经过 SkillGuard 规则扫描",
          Some("自动跑；详情见「技能备份 → 扫描报告」"),
          check_all_scanned),
        b("SK-002", "无技能评分 < 30（自动阻止安装阈值）",
          Some("查看「技能备份」红色标记并移除"),
          check_score_threshold),
        b("SK-003", "无技能含同形异义字（西里尔 a/e/o 等）",
          Some("用 SkillGuard SK-005 重新扫描"),
          check_homoglyph),
        b("SK-004", "无技能含 eval(input) / exec(stdin) 类硬触发",
          Some("禁用或更换技能"),
          check_eval_input),
        b("SK-005", "无技能含 base64 解码后执行（分阶段载荷）",
          Some("禁用或更换技能"),
          check_base64_stage),
        b("SK-006", "无技能含未声明的 socket / subprocess 调用",
          Some("manifest 必须声明 network/exec capabilities"),
          check_socket_subprocess),
        b("SK-007", "技能 manifest 含完整 capabilities 声明",
          Some("manifest 缺 capabilities 字段的视为可疑"),
          check_capabilities_declared),
        b("SK-008", "无技能匹配活跃公告（CVE / GHSA / CW-）",
          Some("查看「安全公告」并升级匹配技能"),
          check_advisory_match),
        b("SK-009", "无技能描述含可疑指令（'always use this tool first'）",
          Some("禁用或更换；INJ-014 模式"),
          check_suspicious_description),
        b("SK-010", "已装技能的 publisher 域名一致",
          Some("@scope/* 应来自对应的 publisher 域名"),
          check_publisher_consistency),
        b("SK-011", "已装技能数量在合理范围内（< 50）",
          Some("减少不必要的技能"),
          check_skill_count),
        b("SK-012", "技能 install_path 在受控目录内（非 /tmp 等）",
          Some("移到 ~/.<agent>/skills/ 或 ~/.agents/skills/"),
          check_install_path),
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
        category: Category::SkillSupplyChain,
        description: desc,
        remediation,
        run_fn,
    })
}

// ──────────────────────────────────────────────────────────────────
// 内部：扫描所有本机技能 + 跑 SkillGuard
// ──────────────────────────────────────────────────────────────────

/// 调用一次 discover_all + 对每个技能跑 SkillGuard scan，返回 (skill, report) 对列表
fn scan_all() -> Vec<(DiscoveredSkill, ScanReport)> {
    let skills = discover_all();
    skills
        .into_iter()
        .map(|s| {
            let report = scan_one(&s);
            (s, report)
        })
        .collect()
}

fn scan_one(skill: &DiscoveredSkill) -> ScanReport {
    let root = Path::new(&skill.source_path);
    let manifest = fs::read_to_string(root.join("SKILL.md"))
        .or_else(|_| fs::read_to_string(root.join("package.json")))
        .unwrap_or_default();

    let mut targets: Vec<(String, String, Context)> = Vec::new();
    collect_targets(root, root, &mut targets, 0);

    let bundle = SkillBundle {
        manifest: &manifest,
        files: targets
            .iter()
            .map(|(n, c, ctx)| (n.as_str(), c.as_str(), *ctx))
            .collect(),
    };
    skill_scanner::scan(&bundle)
}

fn collect_targets(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, String, Context)>,
    depth: u32,
) {
    if depth > 4 || out.len() >= 50 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
            if n.starts_with('.') || n == "node_modules" {
                continue;
            }
        }
        if p.is_dir() {
            collect_targets(root, &p, out, depth + 1);
        } else if p.is_file() {
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else { continue };
            let Ok(meta) = p.metadata() else { continue };
            if meta.len() > 256 * 1024 {
                continue;
            }
            let ctx = if name.ends_with(".md") || name.ends_with(".json") || name.ends_with(".toml")
            {
                Context::Mention
            } else {
                Context::Exec
            };
            if let Ok(content) = fs::read_to_string(&p) {
                let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().into_owned();
                out.push((rel, content, ctx));
            }
        }
    }
}

/// 收集所有 hard_triggers 命中 `rule_id` 的技能名
fn collect_hits_by_hard_trigger(
    reports: &[(DiscoveredSkill, ScanReport)],
    rule_id: &str,
) -> Vec<String> {
    reports
        .iter()
        .filter(|(_, r)| r.hard_triggers.iter().any(|t| t.rule_id == rule_id))
        .map(|(s, _)| s.name.clone())
        .collect()
}

/// 收集所有 weighted findings 命中 `rule_id` 的技能名
fn collect_hits_by_finding(
    reports: &[(DiscoveredSkill, ScanReport)],
    rule_id: &str,
) -> Vec<String> {
    reports
        .iter()
        .filter(|(_, r)| r.findings.iter().any(|f| f.rule_id == rule_id))
        .map(|(s, _)| s.name.clone())
        .collect()
}

// ──────────────────────────────────────────────────────────────────
// 12 项检查实现
// ──────────────────────────────────────────────────────────────────

fn check_all_scanned() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let blocked = reports.iter().filter(|(_, r)| r.blocked).count();
    let passed = reports.len() - blocked;
    (
        CheckOutcome::Pass,
        Some(format!("已扫描 {} 个技能（{} 安全 · {} 被阻止）", reports.len(), passed, blocked)),
    )
}

fn check_score_threshold() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let low: Vec<String> = reports
        .iter()
        .filter(|(_, r)| r.score < 30)
        .map(|(s, r)| format!("{} (score={})", s.name, r.score))
        .collect();
    if low.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个，全部 ≥ 30", reports.len())));
    }
    (CheckOutcome::Fail, Some(low.join(", ")))
}

fn check_homoglyph() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let hits = collect_hits_by_hard_trigger(&reports, "SK-005");
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个，无同形字", reports.len())));
    }
    (
        CheckOutcome::Fail,
        Some(format!(
            "以下 {} 个技能的 manifest/文件中含西里尔同形字（а/е/о 等）：{}",
            hits.len(),
            hits.join(", "),
        )),
    )
}

fn check_eval_input() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let hits = collect_hits_by_hard_trigger(&reports, "SK-001");
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个，无 eval(input)", reports.len())));
    }
    (
        CheckOutcome::Fail,
        Some(format!(
            "以下 {} 个技能的代码文件中含 eval(input)/exec(stdin)：{}",
            hits.len(),
            hits.join(", "),
        )),
    )
}

fn check_base64_stage() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let hits = collect_hits_by_hard_trigger(&reports, "SK-008");
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个，无 base64-stage", reports.len())));
    }
    (
        CheckOutcome::Fail,
        Some(format!(
            "以下 {} 个技能含分阶段载荷模式（base64.b64decode + exec / atob）：{}",
            hits.len(),
            hits.join(", "),
        )),
    )
}

fn check_socket_subprocess() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let hits = collect_hits_by_hard_trigger(&reports, "SK-002");
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个，无未声明 socket/subprocess", reports.len())));
    }
    (
        CheckOutcome::Fail,
        Some(format!(
            "以下 {} 个技能的代码含未声明的 socket.connect / subprocess.Popen / os.system 调用：{}",
            hits.len(),
            hits.join(", "),
        )),
    )
}

fn check_capabilities_declared() -> (CheckOutcome, Option<String>) {
    let skills = discover_all();
    if skills.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let mut missing: Vec<String> = Vec::new();
    for s in &skills {
        let root = Path::new(&s.source_path);
        // 优先看 SKILL.md frontmatter；回退 package.json
        let mut has = false;
        if let Ok(text) = fs::read_to_string(root.join("SKILL.md")) {
            // 极简检测：frontmatter 含 "capabilities:" 或正文 H2 提到 Capabilities
            let lower = text.to_lowercase();
            if lower.contains("capabilities:") || lower.contains("# capabilities") {
                has = true;
            }
        }
        if !has {
            if let Ok(text) = fs::read_to_string(root.join("package.json")) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if v.get("capabilities").is_some() {
                        has = true;
                    }
                }
            }
        }
        if !has {
            missing.push(s.name.clone());
        }
    }
    if missing.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已检查 {} 个", skills.len())));
    }
    (
        CheckOutcome::Warn,
        Some(format!("{}/{} 缺 capabilities：{}", missing.len(), skills.len(),
            truncate_list(&missing))),
    )
}

fn check_advisory_match() -> (CheckOutcome, Option<String>) {
    // ClawHeart 公告订阅尚未接入云端（W17 计划），现阶段只查本地 advisory 表
    let skills = discover_all();
    if skills.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    #[cfg(feature = "storage")]
    {
        let Some(db_path) = dirs::home_dir().map(|h| h.join(".clawheart-v2").join("clawheart.db"))
        else {
            return (CheckOutcome::Skipped, Some("无 home 目录".into()));
        };
        if !db_path.is_file() {
            return (CheckOutcome::Skipped, Some("ClawHeart DB 不存在".into()));
        }
        let conn = match rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            Ok(c) => c,
            Err(_) => return (CheckOutcome::Skipped, Some("DB 不可读".into())),
        };
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM security_advisories WHERE acknowledged = 0", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        if count == 0 {
            return (
                CheckOutcome::Pass,
                Some(format!("已检查 {} 个技能，无未确认公告", skills.len())),
            );
        }
        // 简单匹配：advisory.affects 是 JSON 数组，技能名出现即视为命中
        let mut hits: Vec<String> = Vec::new();
        let mut stmt = match conn.prepare("SELECT advisory_id, affects FROM security_advisories WHERE acknowledged = 0") {
            Ok(s) => s,
            Err(_) => return (CheckOutcome::Pass, Some("已检查 — 公告表 schema 不兼容，跳过匹配".into())),
        };
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1).unwrap_or_default()))
            })
            .ok();
        if let Some(rows) = rows {
            for r in rows.flatten() {
                let (advisory_id, affects_json) = r;
                let affects: Vec<String> = serde_json::from_str(&affects_json).unwrap_or_default();
                for s in &skills {
                    if affects.iter().any(|a| a == &s.name || a == &s.id) {
                        hits.push(format!("{} ↔ {}", s.name, advisory_id));
                    }
                }
            }
        }
        if hits.is_empty() {
            return (
                CheckOutcome::Pass,
                Some(format!("已检查 {} 个技能 vs {} 公告，无匹配", skills.len(), count)),
            );
        }
        return (CheckOutcome::Fail, Some(hits.join(", ")));
    }
    #[cfg(not(feature = "storage"))]
    {
        (CheckOutcome::Skipped, Some("storage feature 未启用，无法匹配公告".into()))
    }
}

fn check_suspicious_description() -> (CheckOutcome, Option<String>) {
    let reports = scan_all();
    if reports.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let hits = collect_hits_by_finding(&reports, "SK-101");
    if hits.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已扫描 {} 个，描述均干净", reports.len())));
    }
    (CheckOutcome::Warn, Some(format!("描述含可疑指令：{}", hits.join(", "))))
}

fn check_publisher_consistency() -> (CheckOutcome, Option<String>) {
    // 解析 SKILL.md frontmatter 的 name 字段：含 @scope/ 则 scope 应与目录路径一致
    let skills = discover_all();
    if skills.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let mut mismatches: Vec<String> = Vec::new();
    for s in &skills {
        let Some(ref n) = Some(&s.name) else { continue };
        if let Some(rest) = n.strip_prefix('@') {
            if let Some((scope, _)) = rest.split_once('/') {
                let path_lower = s.source_path.to_lowercase();
                // 期望 scope 名出现在 source_path 中（粗启发式）
                if !path_lower.contains(&scope.to_lowercase()) {
                    mismatches.push(format!("{} (scope={})", s.name, scope));
                }
            }
        }
    }
    if mismatches.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已检查 {} 个", skills.len())));
    }
    (CheckOutcome::Warn, Some(format!("scope 与路径不一致：{}", mismatches.join(", "))))
}

fn check_skill_count() -> (CheckOutcome, Option<String>) {
    let skills = discover_all();
    let n = skills.len();
    if n == 0 {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    if n < 50 {
        return (CheckOutcome::Pass, Some(format!("已装 {} 个技能", n)));
    }
    if n < 100 {
        return (CheckOutcome::Warn, Some(format!("已装 {} 个技能（≥ 50）— 建议精简", n)));
    }
    (CheckOutcome::Fail, Some(format!("已装 {} 个技能（≥ 100）— 暴露面过大", n)))
}

fn check_install_path() -> (CheckOutcome, Option<String>) {
    let skills = discover_all();
    if skills.is_empty() {
        return (CheckOutcome::Skipped, Some("未发现本机技能".into()));
    }
    let home = dirs::home_dir();
    let mut bad: Vec<String> = Vec::new();
    for s in &skills {
        let p = &s.source_path;
        // 黑名单：/tmp /var/tmp /Users/Shared 等共享/临时目录
        let lower = p.to_lowercase();
        let in_tmp = lower.starts_with("/tmp/")
            || lower.starts_with("/var/tmp/")
            || lower.contains("/users/shared/")
            || lower.starts_with("/private/tmp/");
        let in_home = home
            .as_ref()
            .is_some_and(|h| p.starts_with(&h.to_string_lossy().to_string()));
        if in_tmp || !in_home {
            bad.push(format!("{} → {}", s.name, p));
        }
    }
    if bad.is_empty() {
        return (CheckOutcome::Pass, Some(format!("已检查 {} 个，全部在 ~/", skills.len())));
    }
    (CheckOutcome::Fail, Some(bad.join(", ")))
}

// ──────────────────────────────────────────────────────────────────
// utils
// ──────────────────────────────────────────────────────────────────

fn truncate_list(items: &[String]) -> String {
    if items.len() <= 5 {
        items.join(", ")
    } else {
        format!("{} 等 {} 个", items[..5].join(", "), items.len())
    }
}
