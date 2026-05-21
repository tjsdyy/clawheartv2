/**
 * Agent 维度的应用记录抽屉
 *
 * 列出本 Agent 的所有 batch snapshots（最近 10 个 batch），
 * 按 applied_at desc 显示，支持单条回滚。
 */
import { useMemo } from "react";
import { X, RotateCcw, Loader2, History, CheckCircle2 } from "lucide-react";
import { useQueries } from "@tanstack/react-query";
import { toast } from "sonner";
import type { DiscoveredAgent } from "@/hooks/useAgents";
import {
  useApplyBatches,
  useApplyRealStatus,
  useRollbackSnapshot,
  listBatchSnapshots,
  type SnapshotDto,
  type BatchSummary,
} from "@/hooks/useAgentConfig";
import { useProviderProfiles } from "@/hooks/useProviders";

interface Props {
  agent: DiscoveredAgent;
  onClose: () => void;
}

export function AgentHistoryDrawer({ agent, onClose }: Props) {
  const agentId = `${agent.platform}/${agent.agent_name}`;
  const { data: batches = [] } = useApplyBatches();
  const { data: profiles = [] } = useProviderProfiles();
  const { data: applyReal } = useApplyRealStatus();
  const rollbackSnapshot = useRollbackSnapshot();

  const recent = batches.slice(0, 10);
  const snapshotQueries = useQueries({
    queries: recent.map((b) => ({
      queryKey: ["agent_config", "batch_snapshots", b.batch_id],
      queryFn: () => listBatchSnapshots(b.batch_id),
      staleTime: 60 * 1000,
    })),
  });

  const rows = useMemo(() => {
    const out: Array<{
      snapshot: SnapshotDto;
      batch: BatchSummary;
      profileName: string | null;
    }> = [];
    for (let i = 0; i < recent.length; i++) {
      const ss: SnapshotDto[] = snapshotQueries[i]?.data ?? [];
      const batch = recent[i];
      const profile = profiles.find((p) => p.id === batch.profile_id);
      for (const s of ss) {
        if (s.agent_id === agentId) {
          out.push({
            snapshot: s,
            batch,
            profileName: profile?.name ?? null,
          });
        }
      }
    }
    out.sort((a, b) => b.snapshot.applied_at.localeCompare(a.snapshot.applied_at));
    return out;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    agentId,
    profiles,
    JSON.stringify(snapshotQueries.map((q) => q.data?.length ?? 0)),
    recent.length,
  ]);

  const realEnabled = applyReal?.enabled ?? false;
  const loading = snapshotQueries.some((q) => q.isLoading);

  async function handleRollback(snapshotId: string) {
    const hint = realEnabled
      ? "将真实还原此次变更"
      : "Dry-run 模式：仅还原沙箱内容";
    if (!confirm(`回滚此条记录？\n${hint}`)) return;
    try {
      await rollbackSnapshot.mutateAsync({
        snapshotId,
        dryRun: !realEnabled,
      });
      toast.success("回滚已提交");
    } catch (e) {
      toast.error(`回滚失败：${e}`);
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl max-h-[80vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div>
            <h3 className="text-[14px] font-semibold tracking-tight flex items-center gap-2">
              <History className="w-4 h-4 text-text-muted" />
              {agent.agent_name} · 应用记录
            </h3>
            <div className="text-[11px] text-text-muted mt-0.5 font-mono">
              {rows.length} 次变更
              {!realEnabled && (
                <span className="ml-2 text-amber-600 dark:text-amber-400">
                  · Dry-run 模式
                </span>
              )}
            </div>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Body */}
        <div className="flex-1 overflow-auto p-5">
          {loading && rows.length === 0 ? (
            <div className="text-[12px] text-text-muted py-4 text-center">
              加载中…
            </div>
          ) : rows.length === 0 ? (
            <div className="text-[13px] text-text-muted py-8 text-center border border-dashed border-border-soft rounded-md">
              暂无应用记录
            </div>
          ) : (
            <div className="space-y-2">
              {rows.map(({ snapshot, batch, profileName }) => {
                const rolled = !!snapshot.rolled_back_at;
                return (
                  <div
                    key={snapshot.id}
                    className={`flex items-center gap-3 px-3 py-2.5 rounded-md border ${
                      rolled
                        ? "border-border-soft bg-bg-elev/40"
                        : "border-emerald-500/20 bg-emerald-500/[0.03]"
                    }`}
                  >
                    <span
                      className="w-1.5 h-1.5 rounded-full flex-shrink-0"
                      style={{
                        background: rolled
                          ? "rgb(var(--text-muted))"
                          : "rgb(var(--accent))",
                      }}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 text-[12.5px]">
                        <span className="font-medium">
                          {profileName ?? batch.profile_id?.slice(0, 8) ?? "—"}
                        </span>
                        {!rolled && (
                          <span className="inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0.5 rounded text-emerald-600 dark:text-emerald-400 bg-emerald-500/10">
                            <CheckCircle2 className="w-2.5 h-2.5" />
                            生效中
                          </span>
                        )}
                        {rolled && (
                          <span className="text-[10px] px-1.5 py-0.5 rounded bg-bg-elev2 text-text-muted">
                            已回滚
                          </span>
                        )}
                      </div>
                      <div className="text-[10.5px] text-text-muted font-mono mt-0.5">
                        {snapshot.applied_at}
                        {rolled && ` · 回滚于 ${snapshot.rolled_back_at}`}
                        <span className="mx-1.5">·</span>
                        <span title={snapshot.config_path}>
                          {snapshot.config_path}
                        </span>
                      </div>
                    </div>
                    {!rolled && (
                      <button
                        onClick={() => handleRollback(snapshot.id)}
                        disabled={rollbackSnapshot.isPending}
                        className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-border hover:border-text-muted text-text-dim disabled:opacity-40"
                      >
                        {rollbackSnapshot.isPending ? (
                          <Loader2 className="w-3 h-3 animate-spin" />
                        ) : (
                          <RotateCcw className="w-3 h-3" />
                        )}
                        回滚
                      </button>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Footer */}
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 text-[11px] text-text-muted">
          回滚遵循「设置 → 安全」的实际写入开关：未启用 → 仅还原 dry-run 沙箱；已启用 → 还原真实文件。
        </footer>
      </div>
    </div>
  );
}
