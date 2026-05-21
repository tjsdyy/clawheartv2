import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

// ──────────────────────────────────────────────────────────────────
// 类型（与 Rust DTO 对齐）
// ──────────────────────────────────────────────────────────────────

export type PatchRisk = "Safe" | "Caution" | "Risky";

export type ConfigSource =
  | { type: "JsonFile"; path: string; json_path: string }
  | { type: "TomlFile"; path: string; key: string }
  | { type: "EnvVar"; name: string; scope: string }
  | { type: "VsCodeWorkspace"; path: string; setting: string }
  | { type: "Unknown" };

export interface ProbeResult {
  agent_id: string;
  agent_platform: string;
  agent_name: string;
  current_base_url: string | null;
  current_key_present: boolean;
  config_source: ConfigSource;
  writable: boolean;
  probe_available: boolean;
  warnings: string[];
}

export interface DiffLine {
  kind: " " | "-" | "+";
  text: string;
}

export interface ConfigPatch {
  agent_id: string;
  agent_platform: string;
  agent_name: string;
  source: ConfigSource;
  before: string;
  after: string;
  diff_lines: DiffLine[];
  risk_level: PatchRisk;
}

export interface ApplyOutcome {
  agent_id: string;
  agent_platform: string;
  agent_name: string;
  success: boolean;
  snapshot_id: string | null;
  config_path: string;
  message: string;
  dry_run: boolean;
}

export interface ApplyBatchResult {
  batch_id: string;
  dry_run: boolean;
  outcomes: ApplyOutcome[];
  success_count: number;
  failure_count: number;
}

export interface BatchSummary {
  batch_id: string;
  profile_id: string | null;
  agent_count: number;
  applied_at: string;
  fully_rolled_back: boolean;
}

export interface SnapshotDto {
  id: string;
  batch_id: string;
  agent_id: string;
  agent_platform: string;
  config_path: string;
  applied_at: string;
  rolled_back_at: string | null;
}

export interface RollbackResult {
  batch_id: string | null;
  snapshots_total: number;
  snapshots_restored: number;
  failures: string[];
}

const QK_PROBES = ["agent_config", "probes"] as const;
const QK_BATCHES = ["agent_config", "batches"] as const;
const QK_APPLY_REAL = ["agent_config", "apply_real"] as const;

export interface ApplyRealStatus {
  enabled: boolean;
  forced_dry_run: boolean;
  setting_key: string;
}

export function useApplyRealStatus() {
  return useQuery({
    queryKey: QK_APPLY_REAL,
    queryFn: async (): Promise<ApplyRealStatus> => {
      if (!inTauri) {
        return { enabled: false, forced_dry_run: true, setting_key: "apply_real_enabled" };
      }
      return invoke<ApplyRealStatus>("get_apply_real_status");
    },
    staleTime: 30 * 1000,
  });
}

export function useSetApplyRealEnabled() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      enabled,
      acknowledged,
    }: {
      enabled: boolean;
      acknowledged: boolean;
    }): Promise<ApplyRealStatus> => {
      if (!inTauri) {
        return { enabled, forced_dry_run: !enabled, setting_key: "apply_real_enabled" };
      }
      return invoke<ApplyRealStatus>("set_apply_real_enabled", {
        enabled,
        acknowledged,
      });
    },
    onSuccess: (data) => {
      qc.setQueryData(QK_APPLY_REAL, data);
      qc.invalidateQueries({ queryKey: QK_BATCHES });
    },
    onError: (err) => {
      toast.error(`切换失败：${err}`);
    },
  });
}

// ──────────────────────────────────────────────────────────────────
// Hooks
// ──────────────────────────────────────────────────────────────────

const MOCK_PROBES: ProbeResult[] = [
  {
    agent_id: "cursor/Cursor",
    agent_platform: "cursor",
    agent_name: "Cursor",
    current_base_url: "https://api.openai.com/v1",
    current_key_present: true,
    config_source: { type: "JsonFile", path: "~/Library/Application Support/Cursor/User/settings.json", json_path: "cursor.openaiBaseUrl" },
    writable: true,
    probe_available: true,
    warnings: [],
  },
  {
    agent_id: "claude/Claude Code",
    agent_platform: "claude",
    agent_name: "Claude Code",
    current_base_url: "https://api.anthropic.com",
    current_key_present: true,
    config_source: { type: "JsonFile", path: "~/.claude/settings.json", json_path: "env.ANTHROPIC_BASE_URL" },
    writable: true,
    probe_available: true,
    warnings: [],
  },
];

export function useScanAgentConfigs() {
  return useQuery({
    queryKey: QK_PROBES,
    queryFn: async (): Promise<ProbeResult[]> => {
      if (!inTauri) return MOCK_PROBES;
      return invoke<ProbeResult[]>("scan_agent_configs");
    },
    staleTime: 5 * 1000,
  });
}

export function usePlanOverwrite() {
  return useMutation({
    mutationFn: async ({
      profileId,
      agentIds,
    }: {
      profileId: string;
      agentIds: string[];
    }): Promise<ConfigPatch[]> => {
      if (!inTauri) return [];
      return invoke<ConfigPatch[]>("plan_overwrite", {
        input: { profile_id: profileId, agent_ids: agentIds },
      });
    },
    onError: (err) => {
      toast.error(`计算失败：${err}`);
    },
  });
}

export function useApplyOverwrite() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      profileId,
      patches,
      dryRun,
    }: {
      profileId: string;
      patches: ConfigPatch[];
      dryRun: boolean;
    }): Promise<ApplyBatchResult> => {
      if (!inTauri) {
        return {
          batch_id: "fake-batch",
          dry_run: dryRun,
          outcomes: patches.map((p) => ({
            agent_id: p.agent_id,
            agent_platform: p.agent_platform,
            agent_name: p.agent_name,
            success: true,
            snapshot_id: `fake-snap-${p.agent_id}`,
            config_path: "(浏览器预览)",
            message: "浏览器预览模式",
            dry_run: dryRun,
          })),
          success_count: patches.length,
          failure_count: 0,
        };
      }
      return invoke<ApplyBatchResult>("apply_overwrite", {
        input: { profile_id: profileId, patches, dry_run: dryRun },
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_BATCHES });
    },
    onError: (err) => {
      toast.error(`应用失败：${err}`);
    },
  });
}

export function useApplyBatches() {
  return useQuery({
    queryKey: QK_BATCHES,
    queryFn: async (): Promise<BatchSummary[]> => {
      if (!inTauri) return [];
      return invoke<BatchSummary[]>("list_apply_batches");
    },
    staleTime: 30 * 1000,
  });
}

export async function listBatchSnapshots(batchId: string): Promise<SnapshotDto[]> {
  if (!inTauri) return [];
  return invoke<SnapshotDto[]>("list_batch_snapshots", { batchId });
}

export function useRollbackBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      batchId,
      dryRun,
    }: {
      batchId: string;
      dryRun: boolean;
    }): Promise<RollbackResult> => {
      if (!inTauri) {
        return {
          batch_id: batchId,
          snapshots_total: 0,
          snapshots_restored: 0,
          failures: [],
        };
      }
      return invoke<RollbackResult>("rollback_batch", { batchId, dryRun });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_BATCHES });
    },
    onError: (err) => {
      toast.error(`回滚失败：${err}`);
    },
  });
}

export function useRollbackSnapshot() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      snapshotId,
      dryRun,
    }: {
      snapshotId: string;
      dryRun: boolean;
    }): Promise<RollbackResult> => {
      if (!inTauri) {
        return {
          batch_id: null,
          snapshots_total: 1,
          snapshots_restored: 1,
          failures: [],
        };
      }
      return invoke<RollbackResult>("rollback_snapshot", { snapshotId, dryRun });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_BATCHES });
    },
  });
}
