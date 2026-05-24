//! `clawheart init` — 引导式 onboarding 状态机（Phase 4 实现）
//!
//! 当前为占位 stub，下个 phase 会接入完整状态机：
//!   1. detect-agents
//!   2. choose-tier
//!   3. install-ca (tier2/3)
//!   4. import-providers
//!   5. overwrite-agents
//!   6. start-monitor
//!   7. scan-baseline
//!   8. done

use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

use super::output::{CliResult, Output};

#[derive(Args)]
pub struct InitArgs {
    /// 重置已有引导进度
    #[arg(long)]
    pub reset: bool,
    /// 子命令：step / done
    pub action: Option<String>,
    /// 当前步骤 id（与 action=step 配合）
    pub step_id: Option<String>,
    /// 提交答案
    #[arg(long)]
    pub answer: Option<String>,
}

#[derive(Serialize)]
struct StubStep {
    step: String,
    title: String,
    description: String,
    next_hint: String,
}

pub fn execute(args: InitArgs, json: bool, _db: Option<PathBuf>) -> CliResult {
    let _ = args;
    let stub = StubStep {
        step: "todo".into(),
        title: "引导功能开发中".into(),
        description: "Phase 4 实现完整状态机；当前请使用桌面 GUI 完成引导".into(),
        next_hint: "打开 ClawHeart Desktop GUI 完成首次设置".into(),
    };
    Output::ok_with_text(
        stub,
        "✓ 引导流程将在下个版本接入；当前请使用桌面 GUI 完成 onboarding"
    )
    .emit(json);
    Ok(())
}
