/**
 * 从 Agent 配置文件反向导入渠道
 */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface ChannelCandidate {
  id: string;
  name: string;
  source_agent_id: string;
  source_platform: string;
  base_url: string;
  api_key: string | null;
  protocol: string;
  default_model: string | null;
  provider_kind: string;
  already_exists: boolean;
  warnings: string[];
}

const QK_SCAN = (agentId: string) => ["channel_import", "scan", agentId] as const;

export function useScanImportableChannels(agentId: string | null) {
  return useQuery({
    queryKey: QK_SCAN(agentId ?? ""),
    queryFn: async (): Promise<ChannelCandidate[]> => {
      if (!inTauri || !agentId) return [];
      return invoke<ChannelCandidate[]>("scan_importable_channels", { agentId });
    },
    enabled: !!agentId,
    staleTime: 10_000,
  });
}

export function useImportChannelsBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      candidates,
      assignToAgent,
    }: {
      candidates: ChannelCandidate[];
      assignToAgent?: string;
    }): Promise<string[]> => {
      if (!inTauri) return [];
      return invoke<string[]>("import_channels_batch", {
        candidates,
        assignToAgent: assignToAgent ?? null,
      });
    },
    onSuccess: (created, vars) => {
      qc.invalidateQueries({ queryKey: ["provider_profiles"] });
      qc.invalidateQueries({ queryKey: ["channel_assignments"] });
      qc.invalidateQueries({ queryKey: QK_SCAN(vars.assignToAgent ?? "") });
      if (created.length > 0) {
        toast.success(`成功导入 ${created.length} 个渠道`);
      } else {
        toast.info("没有新渠道被导入（可能都已存在）");
      }
    },
    onError: (err) => toast.error(`导入失败：${err}`),
  });
}
