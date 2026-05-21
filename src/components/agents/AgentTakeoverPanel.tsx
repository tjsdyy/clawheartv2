/**
 * Agent 托管状态面板 —— 紧凑 banner，显示在 Agent tab 下方。
 *
 * 概念：「托管」= ClawHeart 已改写该 Agent 配置文件的 base_url（指向反向代理），
 * 通过 snapshots 表保留原值，可一键还原。
 *
 * 实现复用：现有 apply_overwrite 路径 + rollback_snapshot IPC + applied_batches 历史。
 */
import { useMemo, useState } from "react";
import {
  Zap,
  History,
  RotateCcw,
  Loader2,
  Shield,
} from "lucide-react";
import { useQueries } from "@tanstack/react-query";
import { toast } from "sonner";
import type { DiscoveredAgent } from "@/hooks/useAgents";
import {
  useApplyBatches,
  useApplyRealStatus,
  useRollbackSnapshot,
  listBatchSnapshots,
  type SnapshotDto,
} from "@/hooks/useAgentConfig";
import { useProviderProfiles } from "@/hooks/useProviders";
import { AgentHistoryDrawer } from "./AgentHistoryDrawer";

interface Props {
  agent: DiscoveredAgent;
  /** 当前活跃渠道总数（来自父组件，避免重复查询） */
  profileCount: number;
  /** 已配凭据的可用渠道数 */
  readyProfileCount: number;
}

export function AgentTakeoverPanel({
  agent,
  profileCount,
  readyProfileCount,
}: Props) {
  const agentId = `${agent.platform}/${agent.agent_name}`;
  const { data: batches = [] } = useApplyBatches();
  const { data: profiles = [] } = useProviderProfiles();
  const { data: applyReal } = useApplyRealStatus();
  const rollbackSnapshot = useRollbackSnapshot();
  const [historyOpen, setHistoryOpen] = useState(false);

  // 拉最近 10 batch 的 snapshots，挑出本 agent 的
  const recent = batches.slice(0, 10);
  const snapshotQueries = useQueries({
    queries: recent.map((b) => ({
      queryKey: ["agent_config", "batch_snapshots", b.batch_id],
      queryFn: () => listBatchSnapshots(b.batch_id),
      staleTime: 60 * 1000,
    })),
  });

  // 当前生效 snapshot（最近一次未回滚的）
  const { activeSnapshot, profileName, totalChanges } = useMemo(() => {
    let active: { snapshot: SnapshotDto; profile_id: string | null } | null =
      null;
    let total = 0;
    for (let i = 0; i < recent.length; i++) {
      const ss: SnapshotDto[] = snapshotQueries[i]?.data ?? [];
      for (const s of ss) {
        if (s.agent_id !== agentId) continue;
        total += 1;
        if (!active && !s.rolled_back_at) {
          active = { snapshot: s, profile_id: recent[i].profile_id };
        }
      }
    }
    const profile = active?.profile_id
      ? profiles.find((p) => p.id === active!.profile_id)
      : null;
    return {
      activeSnapshot: active?.snapshot ?? null,
      profileName: profile?.name ?? null,
      totalChanges: total,
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    agentId,
    profiles,
    JSON.stringify(snapshotQueries.map((q) => q.data?.length ?? 0)),
    recent.length,
  ]);

  const taken = !!activeSnapshot;
  const realEnabled = applyReal?.enabled ?? false;

  async function handleDisable() {
    if (!activeSnapshot) return;
    const hint = realEnabled
      ? "将真实还原 Agent 配置文件中的 base_url 等字段"
      : "Dry-run 模式：仅还原 dry-run 沙箱内的配置";
    if (!confirm(`关闭托管？\n${hint}`)) return;
    try {
      await rollbackSnapshot.mutateAsync({
        snapshotId: activeSnapshot.id,
        dryRun: !realEnabled,
      });
      toast.success("已关闭托管，配置文件已还原");
    } catch (e) {
      toast.error(`关闭失败：${e}`);
    }
  }

  return (
    <>
      <div
        className={`mx-5 mt-4 px-4 py-3 rounded-lg border ${
          taken
            ? "border-emerald-500/30 bg-emerald-500/[0.04]"
            : "border-border-soft bg-bg-elev/40"
        }`}
      >
        <div className="flex items-center gap-3">
          {/* Status icon */}
          <div
            className={`w-9 h-9 rounded-md flex items-center justify-center flex-shrink-0 ${
              taken
                ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400"
                : "bg-bg-elev2 text-text-muted"
            }`}
          >
            {taken ? <Zap className="w-4 h-4" /> : <Shield className="w-4 h-4" />}
          </div>

          {/* Status info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-0.5">
              <span className="text-[13px] font-semibold">
                {taken ? "ClawHeart 已托管" : "未托管"}
              </span>
              {taken && (
                <span className="text-[10.5px] px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-medium">
                  通过 ClawHeart 路由
                </span>
              )}
              {!realEnabled && taken && (
                <span className="text-[10.5px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400">
                  Dry-run
                </span>
              )}
            </div>
            <div className="text-[11.5px] text-text-muted font-mono truncate">
              {taken && profileName ? (
                <>
                  生效渠道：
                  <span className="text-text-dim">{profileName}</span>
                  <span className="mx-1.5">·</span>
                  <span>
                    {agent.config_path ?? "未知路径"} → ClawHeart 反代
                  </span>
                </>
              ) : profileCount === 0 ? (
                <>
                  尚未添加任何模型渠道。点击右上角 ⊕ 新增渠道（GLM / Kimi / DeepSeek 等）后即可一键托管。
                </>
              ) : readyProfileCount === 0 ? (
                <>
                  下方 {profileCount} 个渠道均未设 API 凭据。
                  点击「配置凭据」补全后即可一键托管。
                </>
              ) : (
                <>
                  未改写配置文件。点击下方任一渠道的「启用」即可一键托管，
                  Agent 流量将经 ClawHeart 路由 + 审计。
                </>
              )}
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-1.5 flex-shrink-0">
            {totalChanges > 0 && (
              <button
                onClick={() => setHistoryOpen(true)}
                className="flex items-center gap-1 px-2 py-1 text-[11.5px] rounded border border-border hover:border-text-muted text-text-dim"
              >
                <History className="w-3 h-3" />
                历史
                <span className="text-text-muted">{totalChanges}</span>
              </button>
            )}
            {taken && (
              <button
                onClick={handleDisable}
                disabled={rollbackSnapshot.isPending}
                className="flex items-center gap-1 px-2.5 py-1 text-[11.5px] rounded border border-critical/30 text-critical hover:bg-critical/5 disabled:opacity-50"
              >
                {rollbackSnapshot.isPending ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <RotateCcw className="w-3 h-3" />
                )}
                关闭托管
              </button>
            )}
          </div>
        </div>

        {/* Merge 策略提示（仅未托管时） */}
        {!taken && (
          <div className="mt-2 pt-2 border-t border-border-soft/50 text-[10.5px] text-text-muted leading-relaxed">
            💡 ClawHeart 采用 <span className="font-medium">merge</span> 策略改写配置文件：
            仅修改 base_url / api_key 等关键字段，保留 Agent 其他设置，原值持久化备份用于一键还原。
          </div>
        )}
      </div>

      {historyOpen && (
        <AgentHistoryDrawer
          agent={agent}
          onClose={() => setHistoryOpen(false)}
        />
      )}
    </>
  );
}
