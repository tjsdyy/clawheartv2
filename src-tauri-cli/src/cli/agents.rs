//! `clawheart agents` — Agent 发现
use clap::Subcommand;
use serde::Serialize;

use super::output::{CliResult, Output};
use clawheart_lib::agents::scanner::Scanner;
use clawheart_lib::agents::DiscoveredAgent;

#[derive(Subcommand)]
pub enum AgentsCmd {
    /// 列出本机发现的 Agent（Claude Code / Codex / Cursor / Continue / OpenClaw 等）
    List,
    /// 列出某个 Agent 的 MCP server
    Mcp {
        #[arg(long)]
        agent: Option<String>,
    },
    /// 触发重新发现
    Rescan,
}

#[derive(Serialize)]
struct AgentDto {
    id: String,
    platform: String,
    agent_name: String,
    config_path: Option<String>,
    process_name: Option<String>,
    last_seen: String,
    mcp_servers: Vec<String>,
    status: String,
}

impl From<&DiscoveredAgent> for AgentDto {
    fn from(a: &DiscoveredAgent) -> Self {
        Self {
            id: format!("{}/{}", a.platform, a.agent_name),
            platform: a.platform.clone(),
            agent_name: a.agent_name.clone(),
            config_path: a.config_path.clone(),
            process_name: a.process_name.clone(),
            last_seen: a.last_seen.clone(),
            mcp_servers: a.mcp_servers.clone(),
            status: a.status.clone(),
        }
    }
}

pub fn execute(cmd: AgentsCmd, json: bool) -> CliResult {
    let agents = Scanner::with_default_platforms().scan_once();
    let dtos: Vec<AgentDto> = agents.iter().map(AgentDto::from).collect();

    match cmd {
        AgentsCmd::List | AgentsCmd::Rescan => {
            let text = render_agents(&dtos);
            Output::ok_with_text(dtos, text).emit(json);
        }
        AgentsCmd::Mcp { agent } => {
            #[derive(Serialize)]
            struct McpEntry {
                agent_id: String,
                agent_name: String,
                server_name: String,
            }
            let mut entries = Vec::new();
            for a in &dtos {
                if let Some(f) = &agent {
                    if &a.id != f {
                        continue;
                    }
                }
                for srv in &a.mcp_servers {
                    entries.push(McpEntry {
                        agent_id: a.id.clone(),
                        agent_name: a.agent_name.clone(),
                        server_name: srv.clone(),
                    });
                }
            }
            let text = if entries.is_empty() {
                "未发现 MCP server".into()
            } else {
                let mut s = format!("✓ {} 个 MCP server\n", entries.len());
                for e in &entries {
                    s.push_str(&format!("  {} → {}\n", e.agent_id, e.server_name));
                }
                s
            };
            Output::ok_with_text(entries, text).emit(json);
        }
    }
    Ok(())
}

fn render_agents(dtos: &[AgentDto]) -> String {
    if dtos.is_empty() {
        return "未发现任何 Agent".into();
    }
    let mut s = format!("✓ 发现 {} 个 Agent\n\n", dtos.len());
    for a in dtos {
        s.push_str(&format!(
            "  [{}] {} · {} · last_seen={}\n",
            a.id, a.agent_name, a.status, a.last_seen,
        ));
        if let Some(p) = &a.config_path {
            s.push_str(&format!("      config: {}\n", p));
        }
        if !a.mcp_servers.is_empty() {
            s.push_str(&format!("      MCP: {}\n", a.mcp_servers.join(", ")));
        }
    }
    s
}
