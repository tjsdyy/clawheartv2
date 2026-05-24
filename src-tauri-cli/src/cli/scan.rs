//! `clawheart scan` — 跑 80 项 AI 安全审计
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

use super::output::{CliResult, Output};
use clawheart_lib::security::scanner::{run_scan, CheckOutcome, CheckResult};

#[derive(Args)]
pub struct ScanArgs {
    /// 限定类目（FilePermission / McpConfig / CredentialLeak / AgentBehavior /
    /// SkillSupplyChain / SandboxDocker / NetworkExposure / WindowsSpecific）
    #[arg(long)]
    pub category: Option<String>,

    /// 限定 check id（如 FP-001），逗号分隔多个
    #[arg(long, value_delimiter = ',')]
    pub ids: Vec<String>,
}

#[derive(Serialize)]
struct ScanRunDto {
    run_id: Option<i64>,
    started_at: String,
    completed_at: String,
    total: u32,
    passed: u32,
    failed: u32,
    warned: u32,
    skipped: u32,
    results: Vec<CheckResultDto>,
}

#[derive(Serialize)]
struct CheckResultDto {
    id: String,
    category: String,
    outcome: String,
    description: String,
    detail: Option<String>,
    remediation: Option<String>,
}

impl From<&CheckResult> for CheckResultDto {
    fn from(r: &CheckResult) -> Self {
        Self {
            id: r.id.to_string(),
            category: format!("{:?}", r.category),
            outcome: match r.outcome {
                CheckOutcome::Pass => "pass",
                CheckOutcome::Fail => "fail",
                CheckOutcome::Warn => "warn",
                CheckOutcome::Skipped => "skipped",
            }
            .into(),
            description: r.description.to_string(),
            detail: r.detail.clone(),
            remediation: r.remediation.map(String::from),
        }
    }
}

pub fn execute(args: ScanArgs, json: bool, _db: Option<PathBuf>) -> CliResult {
    // 拼接 selected keys：category + ids
    let mut keys: Vec<String> = Vec::new();
    if let Some(c) = &args.category {
        keys.push(c.clone());
    }
    keys.extend(args.ids.iter().cloned());
    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

    let run = run_scan(&key_refs);

    let dto = ScanRunDto {
        run_id: None, // CLI 暂不持久化（Phase 7 接 DB）
        started_at: run.started_at.clone(),
        completed_at: run.completed_at.unwrap_or_default(),
        total: run.total_checks,
        passed: run.passed,
        failed: run.failed,
        warned: run.warned,
        skipped: run.skipped,
        results: run.results.iter().map(CheckResultDto::from).collect(),
    };

    let summary = format_human_summary(&dto);
    Output::ok_with_text(dto, summary).emit(json);
    Ok(())
}

fn format_human_summary(dto: &ScanRunDto) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "✓ 扫描完成 · 共 {} 项 · {} 通过 / {} 严重 / {} 警告 / {} 跳过\n",
        dto.total, dto.passed, dto.failed, dto.warned, dto.skipped
    ));
    s.push_str(&format!("  时间：{} → {}\n\n", dto.started_at, dto.completed_at));

    // 优先列 fail / warn
    let fails: Vec<&CheckResultDto> = dto.results.iter().filter(|r| r.outcome == "fail").collect();
    let warns: Vec<&CheckResultDto> = dto.results.iter().filter(|r| r.outcome == "warn").collect();

    if !fails.is_empty() {
        s.push_str(&format!("⛔ {} 个严重问题：\n", fails.len()));
        for r in &fails {
            s.push_str(&format!("  [{}] {}\n", r.id, r.description));
            if let Some(d) = &r.detail {
                s.push_str(&format!("      → {}\n", d));
            }
            if let Some(rem) = &r.remediation {
                s.push_str(&format!("      修复：{}\n", rem));
            }
        }
        s.push('\n');
    }

    if !warns.is_empty() {
        s.push_str(&format!("⚠ {} 个警告：\n", warns.len()));
        for r in warns.iter().take(10) {
            s.push_str(&format!("  [{}] {}", r.id, r.description));
            if let Some(d) = &r.detail {
                s.push_str(&format!(" — {}", d));
            }
            s.push('\n');
        }
        if warns.len() > 10 {
            s.push_str(&format!("  ... 余 {} 项\n", warns.len() - 10));
        }
    }

    s
}
