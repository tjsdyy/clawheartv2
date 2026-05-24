//! `clawheart skills` — 本机技能发现 / 鉴定 / 备份
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

use super::output::{CliResult, Output};
use crate::skills::discover::{discover_all, DiscoveredSkill};

#[derive(Subcommand)]
pub enum SkillsCmd {
    /// 列出本机所有技能（通配 ~/.<agent>/skills/）
    List,
    /// 鉴定单个技能（用 SkillGuard 规则集跑）
    Scan(ScanSkillArgs),
    /// 打包备份选中技能为 zip
    Backup(BackupArgs),
}

#[derive(Args)]
pub struct ScanSkillArgs {
    /// 技能 ID（来自 `skills list` 的 id 字段）
    pub id: Option<String>,
    /// 鉴定所有技能
    #[arg(long)]
    pub all: bool,
}

#[derive(Args)]
pub struct BackupArgs {
    /// 技能 ID 列表（空 = 全部）
    pub ids: Vec<String>,
    /// 输出 zip 路径
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Serialize)]
struct SkillSummary {
    id: String,
    name: String,
    description: Option<String>,
    version: Option<String>,
    source_agent: String,
    source_path: String,
    file_count: u32,
    total_bytes: u64,
    in_ssot: bool,
    has_skill_md: bool,
}

impl From<&DiscoveredSkill> for SkillSummary {
    fn from(s: &DiscoveredSkill) -> Self {
        Self {
            id: s.id.clone(),
            name: s.name.clone(),
            description: s.description.clone(),
            version: s.version.clone(),
            source_agent: s.source_agent.clone(),
            source_path: s.source_path.clone(),
            file_count: s.file_count,
            total_bytes: s.total_bytes,
            in_ssot: s.in_ssot,
            has_skill_md: s.has_skill_md,
        }
    }
}

pub fn execute(cmd: SkillsCmd, json: bool, _db: Option<PathBuf>) -> CliResult {
    match cmd {
        SkillsCmd::List => list(json),
        SkillsCmd::Scan(args) => scan(args, json),
        SkillsCmd::Backup(args) => backup(args, json),
    }
}

fn list(json: bool) -> CliResult {
    let skills = discover_all();
    let dtos: Vec<SkillSummary> = skills.iter().map(SkillSummary::from).collect();
    let text = render_list(&skills);
    Output::ok_with_text(dtos, text).emit(json);
    Ok(())
}

fn render_list(skills: &[DiscoveredSkill]) -> String {
    if skills.is_empty() {
        return "未发现本机技能（扫描 ~/.<agent>/skills/）".into();
    }
    let mut s = format!("✓ 发现 {} 个技能：\n\n", skills.len());
    for sk in skills {
        s.push_str(&format!(
            "  [{}] {}  .{}{}  {} 文件 · {} KB\n",
            sk.id,
            sk.name,
            sk.source_agent,
            if sk.in_ssot { " (集中库)" } else { "" },
            sk.file_count,
            sk.total_bytes / 1024,
        ));
        if let Some(d) = &sk.description {
            s.push_str(&format!("    {}\n", d));
        }
    }
    s
}

fn scan(args: ScanSkillArgs, json: bool) -> CliResult {
    use crate::security::skill_scanner::{scan as guard_scan, Context, SkillBundle};
    use std::fs;

    let all = discover_all();

    let targets: Vec<&DiscoveredSkill> = if args.all {
        all.iter().collect()
    } else {
        let id = args
            .id
            .as_ref()
            .ok_or_else(|| "需提供 <id> 或 --all".to_string())?;
        let s = all
            .iter()
            .find(|s| &s.id == id)
            .ok_or_else(|| format!("技能 {} 未找到", id))?;
        vec![s]
    };

    #[derive(Serialize)]
    struct ReportDto {
        id: String,
        name: String,
        score: u32,
        blocked: bool,
        hard_triggers: Vec<String>,
        finding_count: usize,
    }

    let mut reports: Vec<ReportDto> = Vec::new();
    for sk in targets {
        let root = std::path::Path::new(&sk.source_path);
        let manifest = fs::read_to_string(root.join("SKILL.md"))
            .or_else(|_| fs::read_to_string(root.join("package.json")))
            .unwrap_or_default();

        let mut files: Vec<(String, String, Context)> = Vec::new();
        collect_files(root, root, &mut files, 0);
        let bundle = SkillBundle {
            manifest: &manifest,
            files: files
                .iter()
                .map(|(n, c, ctx)| (n.as_str(), c.as_str(), *ctx))
                .collect(),
        };
        let r = guard_scan(&bundle);
        reports.push(ReportDto {
            id: sk.id.clone(),
            name: sk.name.clone(),
            score: r.score,
            blocked: r.blocked,
            hard_triggers: r.hard_triggers.iter().map(|t| t.rule_id.clone()).collect(),
            finding_count: r.findings.len(),
        });
    }

    let mut text = format!("✓ 鉴定完成 · {} 个技能\n\n", reports.len());
    for r in &reports {
        let status = if r.blocked {
            "⛔ 阻止"
        } else if r.score < 60 {
            "⚠ 低分"
        } else {
            "✓ 通过"
        };
        text.push_str(&format!(
            "  {} score={} · {} · findings={}{}\n",
            status,
            r.score,
            r.name,
            r.finding_count,
            if r.hard_triggers.is_empty() {
                String::new()
            } else {
                format!(" · 硬触发：{}", r.hard_triggers.join(","))
            },
        ));
    }

    Output::ok_with_text(reports, text).emit(json);
    Ok(())
}

fn collect_files(
    root: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<(String, String, crate::security::skill_scanner::Context)>,
    depth: u32,
) {
    use crate::security::skill_scanner::Context;
    use std::fs;
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
            collect_files(root, &p, out, depth + 1);
        } else if p.is_file() {
            let Ok(meta) = p.metadata() else { continue };
            if meta.len() > 256 * 1024 {
                continue;
            }
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else { continue };
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

fn backup(args: BackupArgs, json: bool) -> CliResult {
    use crate::skills::backup::backup_skills;

    let ids: Vec<String> = if args.ids.is_empty() {
        discover_all().iter().map(|s| s.id.clone()).collect()
    } else {
        args.ids
    };

    if ids.is_empty() {
        return Err("无可备份技能".into());
    }

    let out_path = match args.output {
        Some(p) => p,
        None => {
            let dir = dirs::download_dir()
                .or_else(dirs::home_dir)
                .ok_or_else(|| "无法确定下载目录".to_string())?;
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            dir.join(format!("clawheart-skills-backup-{}.zip", ts))
        }
    };

    let result = backup_skills(&ids, &out_path).map_err(|e| format!("打包失败: {}", e))?;

    let text = format!(
        "✓ 备份完成 · {} 个技能 · {} KB\n  位置：{}",
        result.skill_count,
        result.total_bytes / 1024,
        result.zip_path,
    );
    Output::ok_with_text(result, text).emit(json);
    Ok(())
}
