//! `clawheart init` — 对话式引导状态机。
//!
//! 设计：每个 step 输出 JSON {step, title, description, options, input_type, next_hint}，
//! Agent 解析后用自然语言展现给用户、收集答案、再调 `init step <id> --answer=X` 进下一步。
//!
//! 状态持久化在 `~/.clawheart-v2/init_state.json`，跨进程 / 跨终端续接。
//!
//! Steps（首版）:
//!   1. welcome           — 介绍 ClawHeart
//!   2. detect-agents     — 自动跑 agents list, 报告发现数
//!   3. choose-tier       — 选 tier1（推荐）/ skip-to-gui（tier2/3 复杂操作引导回桌面）
//!   4. setup-base-url    — 告诉用户怎么改 OPENAI_BASE_URL（CLI 不直接动 Agent 配置）
//!   5. scan-baseline     — 跑一次 80 项扫描做基线
//!   6. done              — 摘要 + 下一步建议

use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::output::{CliResult, Output};

// ──────────────────────────────────────────────────────────────────
// CLI 参数
// ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct InitArgs {
    /// 重置已有引导进度
    #[arg(long)]
    pub reset: bool,
    /// 子命令：step / done（留空 = 显示当前 step）
    pub action: Option<String>,
    /// 当前步骤 id（与 action=step 配合）
    pub step_id: Option<String>,
    /// 提交答案
    #[arg(long)]
    pub answer: Option<String>,
}

// ──────────────────────────────────────────────────────────────────
// State
// ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StepId {
    Welcome,
    DetectAgents,
    ChooseTier,
    SetupBaseUrl,
    ScanBaseline,
    Done,
}

impl StepId {
    pub fn as_kebab(&self) -> &'static str {
        match self {
            StepId::Welcome      => "welcome",
            StepId::DetectAgents => "detect-agents",
            StepId::ChooseTier   => "choose-tier",
            StepId::SetupBaseUrl => "setup-base-url",
            StepId::ScanBaseline => "scan-baseline",
            StepId::Done         => "done",
        }
    }

    pub fn from_kebab(s: &str) -> Option<Self> {
        match s {
            "welcome"        => Some(StepId::Welcome),
            "detect-agents"  => Some(StepId::DetectAgents),
            "choose-tier"    => Some(StepId::ChooseTier),
            "setup-base-url" => Some(StepId::SetupBaseUrl),
            "scan-baseline"  => Some(StepId::ScanBaseline),
            "done"           => Some(StepId::Done),
            _                => None,
        }
    }

    /// 步骤顺序（用于 progress / next）
    pub fn order(&self) -> u32 {
        match self {
            StepId::Welcome      => 1,
            StepId::DetectAgents => 2,
            StepId::ChooseTier   => 3,
            StepId::SetupBaseUrl => 4,
            StepId::ScanBaseline => 5,
            StepId::Done         => 6,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            StepId::Welcome      => Some(StepId::DetectAgents),
            StepId::DetectAgents => Some(StepId::ChooseTier),
            StepId::ChooseTier   => Some(StepId::SetupBaseUrl),
            StepId::SetupBaseUrl => Some(StepId::ScanBaseline),
            StepId::ScanBaseline => Some(StepId::Done),
            StepId::Done         => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InitState {
    pub version: u32,
    pub current_step: Option<String>,
    pub answers: serde_json::Map<String, serde_json::Value>,
    pub started_at: Option<String>,
    pub updated_at: Option<String>,
    pub completed: bool,
}

impl InitState {
    pub fn current(&self) -> StepId {
        self.current_step
            .as_deref()
            .and_then(StepId::from_kebab)
            .unwrap_or(StepId::Welcome)
    }

    pub fn set_current(&mut self, step: StepId) {
        self.current_step = Some(step.as_kebab().to_string());
        self.updated_at = Some(now_iso());
        if step == StepId::Done {
            self.completed = true;
        }
    }
}

fn state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".clawheart-v2")
        .join("init_state.json")
}

fn load_state() -> InitState {
    let p = state_path();
    if let Ok(text) = std::fs::read_to_string(&p) {
        if let Ok(s) = serde_json::from_str::<InitState>(&text) {
            return s;
        }
    }
    InitState {
        version: 1,
        current_step: Some(StepId::Welcome.as_kebab().into()),
        answers: serde_json::Map::new(),
        started_at: Some(now_iso()),
        updated_at: Some(now_iso()),
        completed: false,
    }
}

fn save_state(s: &InitState) -> Result<(), String> {
    let p = state_path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {}", e))?;
    }
    let text = serde_json::to_string_pretty(s).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&p, text).map_err(|e| format!("write: {}", e))?;
    Ok(())
}

fn reset_state() -> Result<(), String> {
    let p = state_path();
    let _ = std::fs::remove_file(&p);
    Ok(())
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{}", secs)
}

// ──────────────────────────────────────────────────────────────────
// Step descriptors（每步对外展示的结构）
// ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct StepInfo {
    step: String,
    progress: u32,
    total: u32,
    title: String,
    description: String,
    /// "choice" | "confirm" | "text" | "info-only"
    input_type: String,
    options: Vec<StepOption>,
    default_answer: Option<String>,
    completed: bool,
    next_hint: String,
    /// 附加上下文（如发现的 agent 列表、扫描结果等）
    context: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct StepOption {
    id: String,
    label: String,
    details: String,
    recommended: bool,
}

const TOTAL_STEPS: u32 = 6;

// ──────────────────────────────────────────────────────────────────
// Step 渲染
// ──────────────────────────────────────────────────────────────────

fn build_step_info(step: StepId, state: &InitState) -> StepInfo {
    match step {
        StepId::Welcome => StepInfo {
            step: step.as_kebab().into(),
            progress: step.order(),
            total: TOTAL_STEPS,
            title: "欢迎使用 ClawHeart".into(),
            description: "本机 AI 安全网关 · 让每个 Agent 调用可观察 · 可拦截 · 可解释。\n\
                          接下来 5 步会带你完成基础设置（约 2 分钟）。"
                .into(),
            input_type: "confirm".into(),
            options: vec![
                StepOption {
                    id: "go".into(),
                    label: "继续".into(),
                    details: "进入下一步".into(),
                    recommended: true,
                },
                StepOption {
                    id: "abort".into(),
                    label: "退出".into(),
                    details: "稍后再说，进度已保存".into(),
                    recommended: false,
                },
            ],
            default_answer: Some("go".into()),
            completed: state.completed,
            next_hint: "clawheart init step welcome --answer=go".into(),
            context: None,
        },

        StepId::DetectAgents => {
            // 实时扫描 Agent
            let agents =
                crate::agents::scanner::Scanner::with_default_platforms().scan_once();
            let names: Vec<String> = agents
                .iter()
                .map(|a| format!("{} ({})", a.agent_name, a.platform))
                .collect();
            let ctx = serde_json::json!({
                "count": agents.len(),
                "agents": names,
            });

            let desc = if agents.is_empty() {
                "未在本机发现任何 AI Agent（Claude Code / Codex / Cursor / 等）。\n\
                 若你刚装新工具，先启动一次让 ClawHeart 记录。"
                    .to_string()
            } else {
                format!(
                    "已发现 {} 个 Agent：{}",
                    agents.len(),
                    names.join("、")
                )
            };

            StepInfo {
                step: step.as_kebab().into(),
                progress: step.order(),
                total: TOTAL_STEPS,
                title: "扫描本机 AI Agent".into(),
                description: desc,
                input_type: "confirm".into(),
                options: vec![StepOption {
                    id: "ack".into(),
                    label: "继续".into(),
                    details: "记录这些 Agent，进入下一步".into(),
                    recommended: true,
                }],
                default_answer: Some("ack".into()),
                completed: false,
                next_hint: "clawheart init step detect-agents --answer=ack".into(),
                context: Some(ctx),
            }
        }

        StepId::ChooseTier => StepInfo {
            step: step.as_kebab().into(),
            progress: step.order(),
            total: TOTAL_STEPS,
            title: "选择监控模式".into(),
            description: "三档监控模式覆盖不同强度 — 当前 CLI 引导只配置 tier1（最简）。\n\
                          tier2/tier3 涉及系统级权限，请在桌面 GUI 中完成。"
                .into(),
            input_type: "choice".into(),
            options: vec![
                StepOption {
                    id: "tier1".into(),
                    label: "tier1 · 端点映射（推荐）".into(),
                    details:
                        "Agent 改 OPENAI_BASE_URL 指向 127.0.0.1:19112 · 零侵入、无需证书"
                            .into(),
                    recommended: true,
                },
                StepOption {
                    id: "skip-to-gui".into(),
                    label: "tier2/tier3 → 转到桌面 GUI".into(),
                    details:
                        "系统代理 + 自签 CA / 沙箱隔离 — 需图形界面引导，跳过 CLI 配置"
                            .into(),
                    recommended: false,
                },
            ],
            default_answer: Some("tier1".into()),
            completed: false,
            next_hint: "clawheart init step choose-tier --answer=tier1".into(),
            context: None,
        },

        StepId::SetupBaseUrl => {
            // 根据上一步答案决定文案
            let tier = state
                .answers
                .get("choose-tier")
                .and_then(|v| v.as_str())
                .unwrap_or("tier1");
            let desc = if tier == "tier1" {
                "把以下环境变量设到你的 shell（~/.zshrc 或 ~/.bashrc）：\n\n\
                 export OPENAI_BASE_URL=http://127.0.0.1:19112/v1\n\
                 export ANTHROPIC_BASE_URL=http://127.0.0.1:19112\n\n\
                 设完后用 `source ~/.zshrc` 让它生效。\n\
                 若用 Cursor / Claude Code 等 GUI 工具，在它们的设置里把 base URL 也指到这里。"
                    .to_string()
            } else {
                "你选择了进入桌面 GUI 引导 tier2/3 — 请打开 ClawHeart Desktop，\n\
                 在「监控模式」工具页完成证书安装 / 沙箱设置。"
                    .to_string()
            };

            StepInfo {
                step: step.as_kebab().into(),
                progress: step.order(),
                total: TOTAL_STEPS,
                title: "配置 Agent 指向 ClawHeart 代理".into(),
                description: desc,
                input_type: "confirm".into(),
                options: vec![StepOption {
                    id: "done".into(),
                    label: "已完成".into(),
                    details: "进入扫描基线步骤".into(),
                    recommended: true,
                }],
                default_answer: Some("done".into()),
                completed: false,
                next_hint: "clawheart init step setup-base-url --answer=done".into(),
                context: None,
            }
        }

        StepId::ScanBaseline => {
            // 触发一次扫描作为基线
            let run = crate::security::scanner::run_scan(&[]);
            let ctx = serde_json::json!({
                "total":   run.total_checks,
                "passed":  run.passed,
                "failed":  run.failed,
                "warned":  run.warned,
                "skipped": run.skipped,
            });
            let desc = format!(
                "已完成首次安全扫描作为基线：{} 通过 · {} 严重 · {} 警告 · {} 跳过（总 {} 项）",
                run.passed, run.failed, run.warned, run.skipped, run.total_checks
            );
            StepInfo {
                step: step.as_kebab().into(),
                progress: step.order(),
                total: TOTAL_STEPS,
                title: "建立安全基线".into(),
                description: desc,
                input_type: "confirm".into(),
                options: vec![StepOption {
                    id: "ack".into(),
                    label: "完成引导".into(),
                    details: "结束 init 流程".into(),
                    recommended: true,
                }],
                default_answer: Some("ack".into()),
                completed: false,
                next_hint: "clawheart init step scan-baseline --answer=ack".into(),
                context: Some(ctx),
            }
        }

        StepId::Done => StepInfo {
            step: step.as_kebab().into(),
            progress: step.order(),
            total: TOTAL_STEPS,
            title: "完成 ✓".into(),
            description: "ClawHeart 初始化已就绪。下一步建议：\n\
                          • `clawheart status` — 查看整机状态\n\
                          • `clawheart skills scan --all` — 鉴定本机所有技能\n\
                          • `clawheart agents list` — 重新发现 Agent\n\
                          • 打开 ClawHeart Desktop 查看实时拦截流"
                .into(),
            input_type: "info-only".into(),
            options: vec![],
            default_answer: None,
            completed: true,
            next_hint: "".into(),
            context: Some(serde_json::json!({
                "completed": true,
            })),
        },
    }
}

// ──────────────────────────────────────────────────────────────────
// Execute
// ──────────────────────────────────────────────────────────────────

pub fn execute(args: InitArgs, json: bool, _db: Option<PathBuf>) -> CliResult {
    if args.reset {
        reset_state().map_err(|e| format!("reset: {}", e))?;
    }

    let mut state = load_state();

    // 子命令：step <id> --answer=...
    if args.action.as_deref() == Some("step") {
        let step_id_str = args
            .step_id
            .ok_or_else(|| "缺少 <step_id> 参数".to_string())?;
        let step_id = StepId::from_kebab(&step_id_str)
            .ok_or_else(|| format!("未知 step: {}", step_id_str))?;
        let answer = args
            .answer
            .ok_or_else(|| "缺少 --answer=...".to_string())?;

        // 校验 answer 合法（非 info-only 必须命中 options.id）
        let info = build_step_info(step_id, &state);
        if info.input_type != "info-only" {
            let opt_ids: Vec<&str> = info.options.iter().map(|o| o.id.as_str()).collect();
            if !opt_ids.contains(&answer.as_str()) {
                return Err(format!(
                    "answer 不合法：{} · 允许：{}",
                    answer,
                    opt_ids.join(", ")
                ));
            }
        }

        // 记答案
        state.answers.insert(
            step_id.as_kebab().into(),
            serde_json::Value::String(answer.clone()),
        );

        // 特殊：welcome=abort → 不前进，直接告知
        if step_id == StepId::Welcome && answer == "abort" {
            save_state(&state).ok();
            let info = StepInfo {
                step: "aborted".into(),
                progress: state.current().order(),
                total: TOTAL_STEPS,
                title: "已退出".into(),
                description: "随时可继续：`clawheart init`".into(),
                input_type: "info-only".into(),
                options: vec![],
                default_answer: None,
                completed: false,
                next_hint: "clawheart init".into(),
                context: None,
            };
            Output::ok_with_text(info, "✓ 已退出引导，进度已保存").emit(json);
            return Ok(());
        }

        // 前进
        if let Some(next) = step_id.next() {
            state.set_current(next);
        }
        save_state(&state).map_err(|e| format!("save state: {}", e))?;

        // 输出下一步
        let next_info = build_step_info(state.current(), &state);
        let text = render_step_human(&next_info);
        Output::ok_with_text(next_info, text).emit(json);
        return Ok(());
    }

    // 默认：显示当前 step
    save_state(&state).ok(); // 确保 state file 存在
    let info = build_step_info(state.current(), &state);
    let text = render_step_human(&info);
    Output::ok_with_text(info, text).emit(json);
    Ok(())
}

fn render_step_human(info: &StepInfo) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "[{}/{}] {}\n\n",
        info.progress, info.total, info.title
    ));
    s.push_str(&info.description);
    s.push_str("\n\n");

    if !info.options.is_empty() {
        s.push_str("选项：\n");
        for o in &info.options {
            let star = if o.recommended { " ⭐推荐" } else { "" };
            s.push_str(&format!("  • {} — {}{}\n", o.label, o.details, star));
        }
        s.push('\n');
    }

    if !info.next_hint.is_empty() {
        s.push_str(&format!("下一步：{}\n", info.next_hint));
    }

    s
}

// 让 Path 类型在某处至少被使用，避免 dead-code
#[allow(dead_code)]
fn _path_marker() -> &'static Path {
    Path::new(".")
}
