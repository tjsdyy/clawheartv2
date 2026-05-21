import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";
import { toast } from "sonner";

export interface DiscoveredAgent {
  platform: string;
  agent_name: string;
  config_path: string | null;
  process_name: string | null;
  last_seen: string;
  mcp_servers: string[];
  config_hash: string | null;
  /** active | config_broken | candidate | idle | offline */
  status: string;
  /** 未知平台候选发现时的命中线索；已知平台为 undefined/[] */
  discovery_signals?: string[];
}

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

const MOCK_AGENTS: DiscoveredAgent[] = [
  { platform: "claude", agent_name: "Claude Code", config_path: "~/.claude", process_name: "claude",
    last_seen: "14:31:32", mcp_servers: ["filesystem", "github", "postgres", "@some/mcp"], config_hash: null, status: "active" },
  { platform: "codex", agent_name: "Codex CLI", config_path: "~/.codex", process_name: "codex",
    last_seen: "14:32:01", mcp_servers: [], config_hash: null, status: "active" },
  { platform: "cursor", agent_name: "Cursor", config_path: "~/.cursor", process_name: "Cursor",
    last_seen: "昨天 22:14", mcp_servers: [], config_hash: null, status: "offline" },
];

export function useAgents() {
  return useQuery({
    queryKey: QK.agents,
    queryFn: async () =>
      inTauri ? invoke<DiscoveredAgent[]>("list_agents") : MOCK_AGENTS,
    staleTime: 60_000,
  });
}

export function useRediscoverAgents() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      if (!inTauri) return MOCK_AGENTS;
      return invoke<DiscoveredAgent[]>("discover_agents_now");
    },
    onSuccess: (agents) => {
      qc.setQueryData(QK.agents, agents);
      toast.success(`发现 ${agents.length} 个 Agent`);
    },
    onError: (err) => toast.error(`扫描失败：${err}`),
  });
}

// ──────────────────────────────────────────────────────────────────
// 候选 Agent 决策（confirm/ignore）持久化
// ──────────────────────────────────────────────────────────────────

export function useConfirmUnknownAgent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (platform: string) => {
      if (!inTauri) return;
      await invoke("confirm_unknown_agent", { platform });
    },
    onSuccess: (_, platform) => {
      qc.invalidateQueries({ queryKey: QK.agents });
      toast.success("已纳入管理", { description: platform });
    },
    onError: (err) => toast.error(`确认失败：${err}`),
  });
}

export function useIgnoreUnknownAgent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (platform: string) => {
      if (!inTauri) return;
      await invoke("ignore_unknown_agent", { platform });
    },
    onSuccess: (_, platform) => {
      qc.invalidateQueries({ queryKey: QK.agents });
      toast.success("已忽略", { description: platform });
    },
    onError: (err) => toast.error(`忽略失败：${err}`),
  });
}

export function useResetUnknownAgentDecision() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (platform: string) => {
      if (!inTauri) return;
      await invoke("reset_unknown_agent_decision", { platform });
    },
    onSuccess: (_, platform) => {
      qc.invalidateQueries({ queryKey: QK.agents });
      toast.success("决策已重置", { description: platform });
    },
    onError: (err) => toast.error(`重置失败：${err}`),
  });
}
