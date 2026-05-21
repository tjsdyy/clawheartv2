/**
 * Agent ↔ Channel 分配（N:M）的前端 hooks
 *
 * 数据模型：
 *   渠道（profile）是全局资源；Agent 显式分配哪些渠道为自己服务。
 *   AgentsTool 只显示已分配的渠道；模型渠道库管理全局 CRUD。
 */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

const QK_BY_AGENT = (agentId: string) =>
  ["channel_assignments", "by_agent", agentId] as const;
const QK_ALL = ["channel_assignments", "all"] as const;

export interface AssignmentDto {
  agent_id: string;
  profile_id: string;
}

/** 某 Agent 已分配的所有 profile_id */
export function useAgentChannels(agentId: string | null) {
  return useQuery({
    queryKey: QK_BY_AGENT(agentId ?? ""),
    queryFn: async (): Promise<string[]> => {
      if (!inTauri || !agentId) return [];
      return invoke<string[]>("list_agent_channels", { agentId });
    },
    enabled: !!agentId,
    staleTime: 30_000,
  });
}

/** 全部分配关系（渠道库视图用，显示每个渠道分配给了哪些 Agent） */
export function useAllAssignments() {
  return useQuery({
    queryKey: QK_ALL,
    queryFn: async (): Promise<AssignmentDto[]> => {
      if (!inTauri) return [];
      return invoke<AssignmentDto[]>("list_all_assignments");
    },
    staleTime: 30_000,
  });
}

export function useAssignChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      agentId,
      profileId,
    }: {
      agentId: string;
      profileId: string;
    }): Promise<boolean> => {
      if (!inTauri) return true;
      return invoke<boolean>("assign_channel", { agentId, profileId });
    },
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: QK_BY_AGENT(vars.agentId) });
      qc.invalidateQueries({ queryKey: QK_ALL });
    },
    onError: (err) => toast.error(`分配失败：${err}`),
  });
}

export function useUnassignChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      agentId,
      profileId,
    }: {
      agentId: string;
      profileId: string;
    }): Promise<boolean> => {
      if (!inTauri) return true;
      return invoke<boolean>("unassign_channel", { agentId, profileId });
    },
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: QK_BY_AGENT(vars.agentId) });
      qc.invalidateQueries({ queryKey: QK_ALL });
    },
    onError: (err) => toast.error(`取消分配失败：${err}`),
  });
}

export function useReplaceAgentChannels() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      agentId,
      profileIds,
    }: {
      agentId: string;
      profileIds: string[];
    }): Promise<boolean> => {
      if (!inTauri) return true;
      return invoke<boolean>("replace_agent_channels", {
        agentId,
        profileIds,
      });
    },
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: QK_BY_AGENT(vars.agentId) });
      qc.invalidateQueries({ queryKey: QK_ALL });
    },
    onError: (err) => toast.error(`保存分配失败：${err}`),
  });
}
