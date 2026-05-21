import { useEffect, useRef, useState } from "react";
import { Play, Loader2, ShieldCheck, ShieldAlert, ShieldX, MinusCircle, Activity } from "lucide-react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { cn } from "@/lib/utils";
import { useScanItems, useStartScanRun } from "@/hooks/useScan";
import { useScanHistory } from "@/hooks/useScanHistory";
import { ScanRunDetailDrawer } from "./ScanRunDetailDrawer";
import type { CheckResultItem } from "@/hooks/useScanHistory";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

interface ProgressState {
  total: number;
  index: number;
  lastResults: CheckResultItem[];      // 最近 20 条，倒序累积
  counts: { pass: number; fail: number; warn: number; skipped: number };
  currentLabel: string;                 // 当前正在跑/刚跑完的 check 描述
}

const EMPTY_PROGRESS: ProgressState = {
  total: 0,
  index: 0,
  lastResults: [],
  counts: { pass: 0, fail: 0, warn: 0, skipped: 0 },
  currentLabel: "",
};

export function ScanTool() {
  const { data: items = [] } = useScanItems();
  const { data: history = [] } = useScanHistory();
  const startScan = useStartScanRun();
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const [detailId, setDetailId] = useState<number | null>(null);
  const [progress, setProgress] = useState<ProgressState>(EMPTY_PROGRESS);
  const unlistenRefs = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    if (items.length > 0 && selected.size === 0) {
      setSelected(
        new Set(items.filter((i) => i.category !== "WindowsSpecific").map((i) => i.category)),
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items.length]);

  // 订阅后端进度事件
  useEffect(() => {
    if (!inTauri) return;
    let cancelled = false;
    (async () => {
      const u1 = await listen<{ total: number }>("scan:run_started", (e) => {
        if (cancelled) return;
        setProgress({
          ...EMPTY_PROGRESS,
          total: e.payload.total,
          currentLabel: "开始扫描…",
        });
      });
      const u2 = await listen<{ index: number; total: number; result: CheckResultItem }>(
        "scan:check_done",
        (e) => {
          if (cancelled) return;
          setProgress((prev) => {
            const next = { ...prev };
            next.index = e.payload.index;
            next.total = e.payload.total;
            next.currentLabel = `${e.payload.result.id} · ${e.payload.result.description}`;
            const c = { ...next.counts };
            switch (e.payload.result.outcome) {
              case "pass":    c.pass += 1; break;
              case "fail":    c.fail += 1; break;
              case "warn":    c.warn += 1; break;
              case "skipped": c.skipped += 1; break;
            }
            next.counts = c;
            next.lastResults = [e.payload.result, ...prev.lastResults].slice(0, 20);
            return next;
          });
        },
      );
      unlistenRefs.current = [u1, u2];
    })();
    return () => {
      cancelled = true;
      for (const u of unlistenRefs.current) u();
      unlistenRefs.current = [];
    };
  }, []);

  function toggle(cat: string) {
    const next = new Set(selected);
    if (next.has(cat)) next.delete(cat);
    else next.add(cat);
    setSelected(next);
  }

  async function handleStart() {
    setProgress(EMPTY_PROGRESS);
    try {
      await startScan.mutateAsync(Array.from(selected));
    } catch {
      /* toast in hook */
    }
  }

  const totalChecks = items.reduce((s, i) => s + (selected.has(i.category) ? i.count : 0), 0);
  const lastRun = history[0];
  const pct = progress.total > 0 ? (progress.index / progress.total) * 100 : 0;
  const showProgress = startScan.isPending || (progress.index > 0 && progress.index < progress.total);
  const showResultSummary = startScan.data && !startScan.isPending && progress.index === progress.total;

  return (
    <div className="mx-auto py-8 px-12" style={{ maxWidth: 820 }}>
      <h2 className="text-[22px] font-semibold tracking-tight mb-1">本机 AI 安全扫描</h2>
      <p className="text-[13px] text-text-dim mb-6">
        80 项确定性检查 · 零 LLM 依赖 · 实时进度反馈
      </p>

      <div className="surface p-6 mb-4">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-[13px] font-semibold">选择扫描项</h3>
          <span className="text-[11.5px] text-text-dim font-mono">
            已选 {selected.size} / {items.length} 项 · 共 {totalChecks} 项检查
          </span>
        </div>

        <div className="grid grid-cols-2 gap-1.5">
          {items.map((it) => (
            <label
              key={it.category}
              className="flex items-center gap-2.5 px-3 py-2.5 rounded-lg bg-bg-elev2 border border-border-soft text-[13px] cursor-pointer hover:border-accent transition-colors"
            >
              <input
                type="checkbox"
                checked={selected.has(it.category)}
                onChange={() => toggle(it.category)}
                className="accent-accent"
              />
              <span>{it.label}</span>
              <span className="ml-auto font-mono text-[10.5px] text-text-muted bg-bg px-1.5 py-0.5 rounded-full">
                {it.count}
              </span>
            </label>
          ))}
        </div>

        <div className="text-center mt-6">
          <button
            onClick={handleStart}
            disabled={startScan.isPending || selected.size === 0}
            className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {startScan.isPending ? (
              <>
                <Activity className="w-4 h-4 animate-pulse" />
                扫描中…
              </>
            ) : (
              <>
                <Play className="w-4 h-4" />
                开始扫描
              </>
            )}
          </button>

          {!startScan.isPending && lastRun && !showResultSummary && (
            <div className="mt-3 text-[12px] text-text-dim font-mono">
              最近一次：{lastRun.started_at} · {lastRun.failed} 严重 / {lastRun.warned} 警告 / {lastRun.passed} 通过
            </div>
          )}
        </div>
      </div>

      {/* 实时进度区 */}
      {showProgress && (
        <ProgressPanel progress={progress} pct={pct} />
      )}

      {/* 完成结果总结 */}
      {showResultSummary && startScan.data && (
        <div className="surface px-5 py-4 mb-4 flex items-center gap-4">
          <ShieldCheck className="w-5 h-5 text-accent flex-shrink-0" />
          <div className="flex-1">
            <div className="text-[13px] font-semibold mb-0.5">扫描完成</div>
            <div className="text-[12px] font-mono text-text-dim">
              <span className="text-critical font-bold">{startScan.data.failed}</span> 严重 ·{" "}
              <span className="text-high font-bold">{startScan.data.warned}</span> 警告 ·{" "}
              <span className="text-accent font-bold">{startScan.data.passed}</span> 通过 ·{" "}
              <span className="text-text-muted">{startScan.data.skipped}</span> 跳过
            </div>
          </div>
          {startScan.data.run_id > 0 && (
            <button
              onClick={() => setDetailId(startScan.data!.run_id)}
              className="px-3 py-1.5 bg-bg-elev2 border border-border rounded-md text-[12px] text-text-dim hover:text-accent hover:border-accent transition-colors"
            >
              查看详情
            </button>
          )}
        </div>
      )}

      <div className="surface p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-[13px] font-semibold">历史记录</h3>
          <span className="text-[11.5px] text-text-dim font-mono">
            {history.length > 0 ? `最近 ${Math.min(history.length, 20)} 次` : "暂无历史"}
          </span>
        </div>

        {history.length === 0 && (
          <div className="py-8 text-center text-text-muted text-[12px] font-mono">
            点击上方"开始扫描"完成第一次扫描
          </div>
        )}

        {history.map((h, i) => (
          <div
            key={h.id}
            className="grid gap-4 items-center py-3 text-[13px]"
            style={{
              gridTemplateColumns: "180px 120px 1fr auto",
              borderBottom: i < history.length - 1 ? "1px solid rgb(var(--border-soft))" : "none",
            }}
          >
            <span className="font-mono text-text-dim text-[12px]">{h.started_at}</span>
            <span className="flex gap-1.5 font-mono text-[11.5px] font-bold">
              <span className="text-critical">●●{h.failed}</span>
              <span className="text-high">◐{h.warned}</span>
              <span className="text-accent">✓{h.passed}</span>
            </span>
            <span className="text-text">
              共 {h.total} 项检查
              <small className="block mt-0.5 text-[11px] text-text-muted font-mono">
                {h.completed_at ? `完成于 ${h.completed_at}` : "未完成"}
              </small>
            </span>
            <button
              onClick={() => setDetailId(h.id)}
              className="px-3 py-1 bg-bg-elev2 border border-border rounded-md text-[12px] text-text-dim hover:text-accent hover:border-accent transition-colors"
            >
              查看
            </button>
          </div>
        ))}
      </div>

      <div className="mt-6 rounded-md border border-border-soft bg-bg-elev/30 px-3 py-2 text-[11px] text-text-muted font-mono leading-relaxed">
        <span className="text-text-dim font-semibold">扫描范围：</span>
        <code className="text-text-dim">~/.&lt;agent&gt;/skills/*</code> ·{" "}
        <code className="text-text-dim">~/.agents/skills/*</code> ·{" "}
        <code className="text-text-dim">~/.cursor/extensions/*</code>{" "}
        <span className="opacity-70">+ 系统配置 / 凭据 / 网络 / 沙箱等本机环境</span>
        <span className="block mt-0.5 opacity-60">
          ClawHeart 自身数据目录已排除（FP-008 自检例外）
        </span>
      </div>

      <ScanRunDetailDrawer id={detailId} onClose={() => setDetailId(null)} />
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 实时进度面板
// ──────────────────────────────────────────────────────────────────
function ProgressPanel({ progress, pct }: { progress: ProgressState; pct: number }) {
  const { counts, lastResults, index, total, currentLabel } = progress;
  return (
    <div className="surface px-5 py-4 mb-4 animate-fadein">
      {/* 顶部：当前 check + 计数 */}
      <div className="flex items-center gap-3 mb-3">
        <Loader2 className="w-4 h-4 text-accent animate-spin flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="text-[12.5px] font-mono text-text truncate">{currentLabel || "准备扫描…"}</div>
          <div className="text-[10.5px] font-mono text-text-muted mt-0.5">
            {index} / {total || "?"} · {pct.toFixed(0)}%
          </div>
        </div>
        <CountBadge n={counts.fail} color="rgb(var(--critical))" Icon={ShieldX} label="严重" />
        <CountBadge n={counts.warn} color="rgb(var(--high))" Icon={ShieldAlert} label="警告" />
        <CountBadge n={counts.pass} color="rgb(var(--accent))" Icon={ShieldCheck} label="通过" />
        <CountBadge n={counts.skipped} color="rgb(var(--text-muted))" Icon={MinusCircle} label="跳过" />
      </div>

      {/* 进度条 */}
      <div className="h-1 rounded-full bg-bg-elev2 overflow-hidden mb-3">
        <div
          className="h-full transition-all duration-200 ease-out"
          style={{
            width: `${Math.min(pct, 100)}%`,
            background: "rgb(var(--accent))",
          }}
        />
      </div>

      {/* 实时结果流（最近 20 条） */}
      <div className="border-t border-border-soft pt-2 max-h-[160px] overflow-auto">
        {lastResults.length === 0 ? (
          <div className="text-[10.5px] text-text-muted font-mono py-1">等待第一项结果…</div>
        ) : (
          <ul className="space-y-0.5 text-[10.5px] font-mono">
            {lastResults.map((r, i) => (
              <ResultLine key={`${r.id}-${i}`} r={r} fresh={i === 0} />
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

function CountBadge({
  n,
  color,
  Icon,
  label,
}: {
  n: number;
  color: string;
  Icon: typeof ShieldCheck;
  label: string;
}) {
  return (
    <div
      className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10.5px] font-mono"
      title={label}
      style={{
        background: "color-mix(in srgb, " + color + " 10%, transparent)",
        color,
      }}
    >
      <Icon className="w-3 h-3" />
      <span className="font-semibold tabular-nums">{n}</span>
    </div>
  );
}

function ResultLine({ r, fresh }: { r: CheckResultItem; fresh: boolean }) {
  const { icon: Icon, color } = outcomeMeta(r.outcome);
  return (
    <li
      className={cn(
        "flex items-center gap-2 px-1 py-0.5 rounded transition-colors",
        fresh && "bg-accent/[0.04]",
      )}
    >
      <Icon className="w-3 h-3 flex-shrink-0" style={{ color }} />
      <code className="text-text-muted flex-shrink-0">{r.id}</code>
      <span className="text-text-dim flex-1 truncate">{r.description}</span>
      {r.detail && (
        <span className="text-text-muted truncate max-w-[200px]" title={r.detail}>
          {r.detail}
        </span>
      )}
    </li>
  );
}

function outcomeMeta(outcome: CheckResultItem["outcome"]) {
  switch (outcome) {
    case "fail":    return { icon: ShieldX,      color: "rgb(var(--critical))" };
    case "warn":    return { icon: ShieldAlert,  color: "rgb(var(--high))" };
    case "pass":    return { icon: ShieldCheck,  color: "rgb(var(--accent))" };
    case "skipped": return { icon: MinusCircle,  color: "rgb(var(--text-muted))" };
  }
}
