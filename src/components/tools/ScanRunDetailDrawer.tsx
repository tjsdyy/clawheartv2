import { useMemo, useState } from "react";
import { X, ShieldCheck, ShieldAlert, ShieldX, MinusCircle, Search, ChevronDown, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { useScanRun, type CheckResultItem } from "@/hooks/useScanHistory";

interface Props {
  id: number | null;
  onClose: () => void;
}

const OUTCOMES = ["fail", "warn", "pass", "skipped"] as const;
type Outcome = (typeof OUTCOMES)[number];

const OUTCOME_META: Record<Outcome, { label: string; color: string; Icon: typeof ShieldCheck }> = {
  fail:    { label: "严重", color: "rgb(var(--critical))", Icon: ShieldX },
  warn:    { label: "警告", color: "rgb(var(--high))",     Icon: ShieldAlert },
  pass:    { label: "通过", color: "rgb(var(--accent))",   Icon: ShieldCheck },
  skipped: { label: "跳过", color: "rgb(var(--text-muted))", Icon: MinusCircle },
};

export function ScanRunDetailDrawer({ id, onClose }: Props) {
  const { data, isLoading, error } = useScanRun(id);
  /** 当前选中的 outcome filter 集合；空集 = 显示全部 */
  const [activeOutcomes, setActiveOutcomes] = useState<Set<Outcome>>(new Set());
  const [search, setSearch] = useState("");

  const filtered = useMemo(() => {
    if (!data) return [] as CheckResultItem[];
    let pool = data.results;
    if (activeOutcomes.size > 0) {
      pool = pool.filter((r) => activeOutcomes.has(r.outcome as Outcome));
    }
    if (search.trim()) {
      const q = search.toLowerCase();
      pool = pool.filter(
        (r) =>
          r.id.toLowerCase().includes(q) ||
          r.description.toLowerCase().includes(q) ||
          (r.detail ?? "").toLowerCase().includes(q) ||
          r.category.toLowerCase().includes(q),
      );
    }
    // 排序：fail → warn → pass → skipped；同一组按 id
    const order: Record<string, number> = { fail: 0, warn: 1, pass: 2, skipped: 3 };
    return [...pool].sort((a, b) => {
      const ao = order[a.outcome] ?? 9;
      const bo = order[b.outcome] ?? 9;
      return ao !== bo ? ao - bo : a.id.localeCompare(b.id);
    });
  }, [data, activeOutcomes, search]);

  const counts = useMemo(() => {
    const m: Record<Outcome, number> = { fail: 0, warn: 0, pass: 0, skipped: 0 };
    if (data) for (const r of data.results) m[r.outcome as Outcome] = (m[r.outcome as Outcome] ?? 0) + 1;
    return m;
  }, [data]);

  if (id === null) return null;

  function toggleOutcome(o: Outcome) {
    setActiveOutcomes((prev) => {
      const next = new Set(prev);
      if (next.has(o)) next.delete(o);
      else next.add(o);
      return next;
    });
  }

  return (
    <div className="fixed inset-0 z-40 flex" onClick={onClose}>
      <div className="flex-1 bg-black/40 animate-fadein" />

      <aside
        className="w-[760px] max-w-[92vw] bg-bg border-l border-border shadow-2xl flex flex-col overflow-hidden animate-slidein-right"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <header className="px-5 py-3 border-b border-border-soft flex items-center gap-3 flex-shrink-0">
          <div className="flex-1 min-w-0">
            <div className="text-[14px] font-semibold tracking-tight">扫描 #{id} · 详情</div>
            {data && (
              <div className="text-[10.5px] font-mono text-text-muted truncate mt-0.5">
                {data.started_at}
                {data.completed_at && ` → ${data.completed_at}`} · 共 {data.total} 项
              </div>
            )}
          </div>
          <button
            onClick={onClose}
            className="w-7 h-7 rounded-md hover:bg-bg-elev/60 flex items-center justify-center text-text-muted hover:text-text"
          >
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Sticky 顶部筛选条 */}
        {data && (
          <div className="px-5 py-2.5 border-b border-border-soft sticky top-0 z-10 bg-bg/85 backdrop-blur-sm flex items-center gap-2 flex-wrap">
            {/* segmented control 风格的 outcome 筛选 */}
            <div className="flex items-center gap-0.5 p-0.5 bg-bg-elev/40 rounded-lg">
              <SegmentChip
                label="全部"
                count={data.total}
                active={activeOutcomes.size === 0}
                onClick={() => setActiveOutcomes(new Set())}
              />
              {OUTCOMES.map((o) => {
                const meta = OUTCOME_META[o];
                return (
                  <SegmentChip
                    key={o}
                    label={meta.label}
                    count={counts[o]}
                    color={meta.color}
                    Icon={meta.Icon}
                    active={activeOutcomes.has(o)}
                    onClick={() => toggleOutcome(o)}
                    dim={counts[o] === 0}
                  />
                );
              })}
            </div>

            <div className="ml-auto relative w-44">
              <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3 h-3 text-text-muted" />
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="搜索 ID / 描述 / 详情…"
                className="w-full bg-bg-elev/40 border border-border-soft rounded-md pl-6 pr-2 py-1 text-[11.5px] outline-none focus:border-accent transition-colors"
              />
            </div>
          </div>
        )}

        {/* Body */}
        <div className="flex-1 overflow-auto px-5 py-2">
          {isLoading && (
            <div className="text-center py-10 text-text-muted text-[12px]">加载中…</div>
          )}
          {error && (
            <div className="text-center py-10 text-critical text-[12.5px]">{String(error)}</div>
          )}
          {data && filtered.length === 0 && (
            <div className="text-center py-10 text-text-muted text-[12px]">
              当前过滤无结果
            </div>
          )}
          {data && (
            <div className="divide-y divide-border-soft">
              {filtered.map((r) => (
                <CheckRow key={r.id} item={r} />
              ))}
            </div>
          )}
        </div>
      </aside>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Segment Chip
// ──────────────────────────────────────────────────────────────────
function SegmentChip({
  label,
  count,
  color,
  Icon,
  active,
  onClick,
  dim,
}: {
  label: string;
  count: number;
  color?: string;
  Icon?: typeof ShieldCheck;
  active: boolean;
  onClick: () => void;
  dim?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11.5px] font-medium transition-all",
        active
          ? "bg-bg text-text shadow-sm"
          : "text-text-muted hover:text-text",
        dim && !active && "opacity-50",
      )}
      style={active && color ? { color } : undefined}
    >
      {Icon && <Icon className="w-3 h-3" />}
      <span>{label}</span>
      <span className="font-mono opacity-70">{count}</span>
    </button>
  );
}

// ──────────────────────────────────────────────────────────────────
// 紧凑行：默认一行 · 有 detail/remediation 才显示 chevron · 点击展开
// ──────────────────────────────────────────────────────────────────
function CheckRow({ item }: { item: CheckResultItem }) {
  const meta = OUTCOME_META[item.outcome as Outcome] ?? OUTCOME_META.skipped;
  const Icon = meta.Icon;
  const [open, setOpen] = useState(false);
  const hasMore =
    !!item.detail || (!!item.remediation && item.outcome !== "skipped");

  return (
    <div
      className={cn(
        "py-1.5 px-1 transition-colors",
        item.outcome === "fail" && "bg-critical/[0.04]",
        item.outcome === "warn" && "bg-amber-500/[0.03]",
      )}
    >
      <button
        onClick={() => hasMore && setOpen((o) => !o)}
        disabled={!hasMore}
        className={cn(
          "w-full flex items-center gap-2 text-left",
          hasMore && "hover:bg-bg-elev/30 rounded px-1 py-0.5 -mx-1",
        )}
      >
        <Icon className="w-3.5 h-3.5 flex-shrink-0" style={{ color: meta.color }} />
        <code className="font-mono text-[11px] text-text-muted flex-shrink-0 w-14">
          {item.id}
        </code>
        <span
          className="text-[10px] font-mono uppercase tracking-wider flex-shrink-0 w-9"
          style={{ color: meta.color }}
        >
          {meta.label}
        </span>
        <span className="text-[12px] text-text flex-1 truncate" title={item.description}>
          {item.description}
        </span>
        {hasMore &&
          (open ? (
            <ChevronDown className="w-3 h-3 text-text-muted flex-shrink-0" />
          ) : (
            <ChevronRight className="w-3 h-3 text-text-muted flex-shrink-0" />
          ))}
      </button>

      {open && hasMore && (
        <div className="ml-[88px] mt-1 mb-1 space-y-1 animate-fadein">
          {item.detail && (
            <div className="text-[11px] text-text-dim font-mono leading-relaxed break-words rounded bg-bg-elev/40 px-2 py-1">
              {item.detail}
            </div>
          )}
          {item.remediation && item.outcome !== "skipped" && (
            <div className="text-[11px] text-text-muted italic">
              修复：{item.remediation}
            </div>
          )}
          <div className="text-[10px] font-mono text-text-muted opacity-70">
            类目：{item.category}
          </div>
        </div>
      )}
    </div>
  );
}
