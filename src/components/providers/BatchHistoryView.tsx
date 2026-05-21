import { useEffect, useState } from "react";
import {
  History,
  Loader2,
  RotateCcw,
  ChevronRight,
  ChevronDown,
  CheckCircle2,
  CircleDot,
  XCircle,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import {
  useApplyBatches,
  useRollbackBatch,
  useRollbackSnapshot,
  useApplyRealStatus,
  listBatchSnapshots,
  type BatchSummary,
  type SnapshotDto,
} from "@/hooks/useAgentConfig";

export function BatchHistoryView() {
  const { data: batches = [], isLoading, refetch } = useApplyBatches();
  const { data: applyReal } = useApplyRealStatus();
  const dryRunForced = !(applyReal?.enabled ?? false);

  return (
    <div className="p-8 max-w-4xl">
      <header className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-[20px] font-semibold tracking-tight flex items-center gap-2">
            <History className="w-5 h-5 text-text-dim" />
            应用记录
          </h2>
          <p className="text-[12px] text-text-muted mt-0.5">
            列出所有「自动应用到 Agent」操作；每一次可整体或逐项回滚。
          </p>
        </div>
        <button
          onClick={() => refetch()}
          className="text-[12px] px-2.5 py-1.5 rounded-md border border-border hover:border-text-muted text-text-dim"
        >
          刷新
        </button>
      </header>

      {dryRunForced && (
        <div className="mb-4 px-3 py-2.5 rounded-md bg-bg-elev border border-border text-[11.5px] text-text-dim leading-relaxed">
          🛡️ 当前为 dry-run 模式。回滚操作也只会写入沙箱目录；不会还原真实 Agent 配置。
          在「设置 → 安全」开启实际写入后才能影响真实文件。
        </div>
      )}

      {isLoading && (
        <div className="flex items-center gap-2 text-[12.5px] text-text-muted py-4">
          <Loader2 className="w-4 h-4 animate-spin" />
          加载记录中…
        </div>
      )}

      {!isLoading && batches.length === 0 && (
        <div className="text-[13px] text-text-muted px-6 py-10 text-center border border-border-soft rounded-md">
          尚无应用记录。在「模型管理」选中模型渠道后点击「自动应用到 Agent」即可创建第一条。
        </div>
      )}

      <div className="space-y-2">
        {batches.map((b) => (
          <BatchCard key={b.batch_id} batch={b} />
        ))}
      </div>
    </div>
  );
}

function BatchCard({ batch }: { batch: BatchSummary }) {
  const [expanded, setExpanded] = useState(false);
  const [snapshots, setSnapshots] = useState<SnapshotDto[]>([]);
  const [loadingSnapshots, setLoadingSnapshots] = useState(false);
  const rollbackBatch = useRollbackBatch();
  const rollbackSnapshot = useRollbackSnapshot();
  const { data: applyReal } = useApplyRealStatus();
  const realEnabled = applyReal?.enabled ?? false;

  useEffect(() => {
    if (expanded && snapshots.length === 0) {
      (async () => {
        setLoadingSnapshots(true);
        try {
          const r = await listBatchSnapshots(batch.batch_id);
          setSnapshots(r);
        } catch (e) {
          toast.error(`加载快照失败：${e}`);
        } finally {
          setLoadingSnapshots(false);
        }
      })();
    }
  }, [expanded, batch.batch_id, snapshots.length]);

  async function handleRollbackBatch() {
    const note = realEnabled
      ? "将真实还原此次操作的所有修改"
      : "Dry-run 模式：仅还原 dry-run 沙箱内容";
    if (!confirm(`确认回滚整条记录？\n${note}`)) return;
    const r = await rollbackBatch.mutateAsync({
      batchId: batch.batch_id,
      dryRun: !realEnabled,
    });
    if (r.failures.length === 0) {
      toast.success(`回滚成功（${r.snapshots_restored}/${r.snapshots_total}）`);
    } else {
      toast.warning(
        `${r.snapshots_restored} 成功 / ${r.failures.length} 失败`,
      );
    }
    // 重新拉快照展示新状态
    const updated = await listBatchSnapshots(batch.batch_id);
    setSnapshots(updated);
  }

  async function handleRollbackOne(snapshot_id: string) {
    if (!confirm(`回滚此条 snapshot？${realEnabled ? "" : "（dry-run 模式）"}`)) return;
    const r = await rollbackSnapshot.mutateAsync({
      snapshotId: snapshot_id,
      dryRun: !realEnabled,
    });
    if (r.failures.length === 0) {
      toast.success("已回滚");
    } else {
      toast.error(r.failures[0]);
    }
    const updated = await listBatchSnapshots(batch.batch_id);
    setSnapshots(updated);
  }

  const allRolledBack = batch.fully_rolled_back;

  return (
    <div
      className={cn(
        "rounded-md border bg-bg-elev",
        allRolledBack ? "border-border-soft" : "border-border",
      )}
    >
      {/* Batch header */}
      <div className="flex items-center gap-3 px-3.5 py-2.5">
        <button
          onClick={() => setExpanded((v) => !v)}
          className="flex items-center text-text-muted hover:text-text"
        >
          {expanded ? (
            <ChevronDown className="w-4 h-4" />
          ) : (
            <ChevronRight className="w-4 h-4" />
          )}
        </button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-[12.5px] font-medium text-text font-mono">
              {batch.batch_id.slice(0, 18)}…
            </span>
            <span className="text-[11px] text-text-muted">
              {batch.applied_at}
            </span>
            {allRolledBack && (
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-text-muted/15 text-text-muted">
                已全部回滚
              </span>
            )}
          </div>
          <div className="text-[11px] text-text-dim mt-0.5">
            {batch.agent_count} Agent
            {batch.profile_id && (
              <span className="ml-2 text-text-muted">
                · 渠道: <code className="font-mono text-[10.5px]">{batch.profile_id.slice(0, 12)}…</code>
              </span>
            )}
          </div>
        </div>
        <button
          onClick={handleRollbackBatch}
          disabled={allRolledBack || rollbackBatch.isPending}
          className={cn(
            "flex items-center gap-1 px-2.5 py-1 rounded-md text-[11.5px] border transition-colors",
            allRolledBack
              ? "border-border-soft text-text-muted cursor-not-allowed"
              : "border-critical/30 text-critical hover:bg-critical/5",
          )}
        >
          {rollbackBatch.isPending ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <RotateCcw className="w-3 h-3" />
          )}
          回滚整批
        </button>
      </div>

      {/* Snapshots */}
      {expanded && (
        <div className="border-t border-border-soft">
          {loadingSnapshots && (
            <div className="px-4 py-3 flex items-center gap-2 text-[11.5px] text-text-muted">
              <Loader2 className="w-3 h-3 animate-spin" />
              加载快照…
            </div>
          )}
          {!loadingSnapshots && snapshots.length === 0 && (
            <div className="px-4 py-3 text-[11.5px] text-text-muted">
              此条记录无快照
            </div>
          )}
          <div className="divide-y divide-border-soft">
            {snapshots.map((s) => (
              <SnapshotRow
                key={s.id}
                snapshot={s}
                onRollback={() => handleRollbackOne(s.id)}
                busy={rollbackSnapshot.isPending}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function SnapshotRow({
  snapshot,
  onRollback,
  busy,
}: {
  snapshot: SnapshotDto;
  onRollback: () => void;
  busy: boolean;
}) {
  const rolled = !!snapshot.rolled_back_at;
  return (
    <div className="flex items-center gap-3 px-4 py-2 text-[11.5px]">
      {rolled ? (
        <CheckCircle2 className="w-3.5 h-3.5 text-text-muted flex-shrink-0" />
      ) : (
        <CircleDot className="w-3.5 h-3.5 text-emerald-500 flex-shrink-0" />
      )}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="font-medium text-text">{snapshot.agent_platform}</span>
          <span className="text-text-muted">{snapshot.agent_id}</span>
          {rolled && (
            <span className="text-[10px] text-text-muted">
              · 已回滚 {snapshot.rolled_back_at?.slice(11, 19)}
            </span>
          )}
        </div>
        <div className="text-[10.5px] font-mono text-text-muted truncate">
          {snapshot.config_path}
        </div>
      </div>
      <button
        onClick={onRollback}
        disabled={rolled || busy}
        className={cn(
          "text-[10.5px] px-2 py-0.5 rounded border",
          rolled
            ? "border-border-soft text-text-muted cursor-not-allowed"
            : "border-border hover:border-text-muted text-text-dim",
        )}
      >
        {busy ? (
          <Loader2 className="w-3 h-3 animate-spin" />
        ) : rolled ? (
          <XCircle className="w-3 h-3" />
        ) : (
          "回滚此条"
        )}
      </button>
    </div>
  );
}
