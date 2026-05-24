//! ClawHeart CLI — clap 命令解析与分发。
//!
//! 设计原则：
//! - **JSON-first**：所有命令支持 `--json` 输出，给 AI Agent 解析
//! - **共享数据**：与 Desktop GUI 共用 `~/.clawheart-v2/clawheart.db`（SQLite WAL + IMMEDIATE 事务）
//! - **零 GUI 依赖**：不引 tauri / hudsucker / axum，体积 < 10MB

use clap::{Parser, Subcommand};

pub mod agents;
pub mod init;
pub mod output;
pub mod providers;
pub mod scan;
pub mod skills;
pub mod status;

use output::Output;

/// ClawHeart 命令行 — 让任意 AI Agent 通过对话框驱动 ClawHeart 的扫描 / 鉴定 / 治理 / 引导。
#[derive(Parser)]
#[command(name = "clawheart", version, about, long_about = None)]
pub struct Cli {
    /// 输出 JSON（机器可读，AI Agent 默认用这个）
    #[arg(long, global = true)]
    pub json: bool,

    /// 不输出 ANSI 颜色（CI / 管道）
    #[arg(long, global = true)]
    pub no_color: bool,

    /// 自定义 DB 路径
    #[arg(long, global = true, value_name = "PATH")]
    pub db: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// 80 项 AI 安全审计
    Scan(scan::ScanArgs),

    /// 本机技能发现 / 鉴定 / 备份
    Skills {
        #[command(subcommand)]
        cmd: skills::SkillsCmd,
    },

    /// Agent 发现 / MCP 列表
    Agents {
        #[command(subcommand)]
        cmd: agents::AgentsCmd,
    },

    /// Provider Profile 管理 + Agent 一键覆盖
    Providers {
        #[command(subcommand)]
        cmd: providers::ProvidersCmd,
    },

    /// 整机状态摘要：监控模式 / 代理 / DB / 最近扫描
    Status,

    /// 引导设置（首次安装、对话式状态机）
    Init(init::InitArgs),
}

/// CLI 入口（由 `src/bin/clawheart-cli.rs` 调用）
pub fn run() -> i32 {
    let cli = Cli::parse();

    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }

    let result = match cli.command {
        Command::Scan(args)         => scan::execute(args, cli.json, cli.db.clone()),
        Command::Skills { cmd }     => skills::execute(cmd, cli.json, cli.db.clone()),
        Command::Agents { cmd }     => agents::execute(cmd, cli.json),
        Command::Providers { cmd }  => providers::execute(cmd, cli.json, cli.db.clone()),
        Command::Status             => status::execute(cli.json, cli.db.clone()),
        Command::Init(args)         => init::execute(args, cli.json, cli.db.clone()),
    };

    match result {
        Ok(()) => 0,
        Err(e) => {
            Output::error(&e.to_string()).emit(cli.json);
            1
        }
    }
}
