import { useMemo, useState } from "react";
import {
  Search, RotateCcw, ChevronRight, AlertTriangle, X, Loader2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import {
  useSecurityRules,
  useToggleRule,
  useSetRuleAction,
  useResetRule,
  useResetRuleKind,
  KIND_META,
  ACTION_META,
  type RuleKind,
  type SecurityRuleRow,
  type ActionOverride,
} from "@/hooks/useSecurityRules";

type View = RuleKind | "all";

export function SecurityRulesPanel() {
  const { data: rules = [], isLoading } = useSecurityRules();
  const toggle = useToggleRule();
  const setAction = useSetRuleAction();
  const reset = useResetRule();
  const resetKind = useResetRuleKind();

  const [view, setView] = useState<View>("danger");
  const [search, setSearch] = useState("");
  const [showDisabledOnly, setShowDisabledOnly] = useState(false);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [confirmDisable, setConfirmDisable] = useState<SecurityRuleRow | null>(null);

  // 按 kind 分桶
  const buckets = useMemo(() => {
    const m: Record<RuleKind, { total: number; enabled: number; hits: number }> = {
      danger: { total: 0, enabled: 0, hits: 0 },
      injection: { total: 0, enabled: 0, hits: 0 },
      credential: { total: 0, enabled: 0, hits: 0 },
      skill: { total: 0, enabled: 0, hits: 0 },
      audit: { total: 0, enabled: 0, hits: 0 },
    };
    for (const r of rules) {
      m[r.kind].total += 1;
      if (r.enabled) m[r.kind].enabled += 1;
      m[r.kind].hits += r.hits_7d;
    }
    return m;
  }, [rules]);

  const filtered = useMemo(() => {
    let pool = rules;
    if (view !== "all") pool = pool.filter((r) => r.kind === view);
    if (showDisabledOnly) pool = pool.filter((r) => !r.enabled);
    if (search.trim()) {
      const q = search.toLowerCase();
      pool = pool.filter(
        (r) =>
          r.id.toLowerCase().includes(q) ||
          r.description.toLowerCase().includes(q) ||
          (r.category ?? "").toLowerCase().includes(q) ||
          (r.pattern_hint ?? "").toLowerCase().includes(q),
      );
    }
    return pool;
  }, [rules, view, showDisabledOnly, search]);

  const currentKindMeta = view === "all" ? null : KIND_META[view];
  const currentBucket = view === "all" ? null : buckets[view];

  function handleToggleClick(rule: SecurityRuleRow) {
    // 已启用 → 想禁用 且 默认动作是 HardBlock/Block → 二次确认
    if (rule.enabled && (rule.default_action === "hard_block" || rule.default_action === "block")) {
      setConfirmDisable(rule);
      return;
    }
    toggle.mutate({ ruleKind: rule.kind, ruleId: rule.id, enabled: !rule.enabled });
  }

  return (
    <div className="grid h-full" style={{ gridTemplateColumns: "200px 1fr" }}>
      {/* 左：kind 子分类 */}
      <aside className="border-r border-border-soft p-2 overflow-auto">
        <div className="text-[10px] text-text-muted uppercase tracking-wider font-bold font-mono px-2 mb-2">
          规则分组
        </div>
        <KindRow
          label="全部"
          icon="∑"
          total={rules.length}
          enabled={rules.filter((r) => r.enabled).length}
          active={view === "all"}
          onClick={() => setView("all")}
        />
        {(["danger", "injection", "credential", "skill", "audit"] as RuleKind[]).map((k) => {
          const meta = KIND_META[k];
          const b = buckets[k];
          return (
            <KindRow
              key={k}
              label={meta.label}
              icon={meta.icon}
              total={b.total}
              enabled={b.enabled}
              active={view === k}
              onClick={() => setView(k)}
            />
          );
        })}

        <div className="mt-4 px-2 text-[10px] text-text-muted leading-relaxed">
          KillSwitch · L7 应急熔断由「监控」工具页独立管理，不可在此禁用。
        </div>
      </aside>

      {/* 右：列表 */}
      <section className="flex flex-col overflow-hidden">
        {/* 工具条 */}
        <div className="px-5 py-3 flex items-center gap-2 border-b border-border-soft flex-shrink-0">
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <h2 className="text-[14px] font-semibold tracking-tight flex-shrink-0">
              {currentKindMeta?.label ?? "全部规则"}
            </h2>
            {currentBucket && (
              <span className="text-[11.5px] font-mono text-text-muted">
                · {currentBucket.total} 条 · 启用 {currentBucket.enabled}/{currentBucket.total}
                {currentBucket.hits > 0 && ` · 7 天命中 ${currentBucket.hits}`}
              </span>
            )}
          </div>

          <div className="relative max-w-xs">
            <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3 h-3 text-text-muted" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索 ID / 描述 / 模式…"
              className="w-56 bg-bg-elev/40 border border-border-soft rounded-md pl-6 pr-2 py-1 text-[11.5px] outline-none focus:border-accent transition-colors"
            />
          </div>

          <label className="flex items-center gap-1.5 text-[11.5px] text-text-dim cursor-pointer px-2">
            <input
              type="checkbox"
              checked={showDisabledOnly}
              onChange={(e) => setShowDisabledOnly(e.target.checked)}
              className="accent-accent w-3 h-3"
            />
            仅显示已禁用
          </label>

          {view !== "all" && (
            <button
              onClick={() => resetKind.mutate(view)}
              disabled={resetKind.isPending}
              className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-border-soft text-text-muted hover:border-text-muted hover:text-text disabled:opacity-50"
              title="恢复此分组的全部默认设置"
            >
              <RotateCcw className="w-3 h-3" />
              重置分组
            </button>
          )}
        </div>

        {/* 列表 */}
        <div className="flex-1 overflow-auto px-5 py-3 space-y-1">
          {isLoading && (
            <div className="text-center py-10 text-text-muted text-[12px]">加载中…</div>
          )}
          {!isLoading && filtered.length === 0 && (
            <EmptyState />
          )}
          {filtered.map((r) => (
            <RuleItem
              key={`${r.kind}::${r.id}`}
              rule={r}
              expanded={expanded === `${r.kind}::${r.id}`}
              onToggleExpand={() =>
                setExpanded(expanded === `${r.kind}::${r.id}` ? null : `${r.kind}::${r.id}`)
              }
              onToggleEnable={() => handleToggleClick(r)}
              onSetAction={(action) =>
                setAction.mutate({ ruleKind: r.kind, ruleId: r.id, action })
              }
              onReset={() => reset.mutate({ ruleKind: r.kind, ruleId: r.id })}
              busy={toggle.isPending || setAction.isPending || reset.isPending}
            />
          ))}
        </div>
      </section>

      {/* 二次确认 Dialog */}
      {confirmDisable && (
        <ConfirmDisableDialog
          rule={confirmDisable}
          onCancel={() => setConfirmDisable(null)}
          onConfirm={() => {
            toggle.mutate({
              ruleKind: confirmDisable.kind,
              ruleId: confirmDisable.id,
              enabled: false,
            });
            setConfirmDisable(null);
          }}
        />
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
function KindRow({
  label, icon, total, enabled, active, onClick,
}: {
  label: string; icon: string; total: number; enabled: number; active: boolean; onClick: () => void;
}) {
  const partial = enabled < total;
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] mt-0.5 transition-colors",
        active
          ? "bg-bg-elev/70 text-text"
          : "text-text-dim hover:text-text hover:bg-bg-elev/40",
      )}
    >
      <span className="text-[14px] flex-shrink-0">{icon}</span>
      <span className="flex-1 text-left truncate">{label}</span>
      <span
        className={cn(
          "text-[10.5px] font-mono tabular-nums",
          partial ? "text-amber-500" : "text-text-muted",
        )}
      >
        {enabled}/{total}
      </span>
    </button>
  );
}

function RuleItem({
  rule, expanded, onToggleExpand, onToggleEnable, onSetAction, onReset, busy,
}: {
  rule: SecurityRuleRow;
  expanded: boolean;
  onToggleExpand: () => void;
  onToggleEnable: () => void;
  onSetAction: (action: ActionOverride) => void;
  onReset: () => void;
  busy: boolean;
}) {
  const actionMeta = ACTION_META[rule.default_action];
  const effectiveAction = rule.action_override ?? null;

  return (
    <div
      className={cn(
        "rounded-md border transition-colors",
        rule.enabled
          ? "border-border-soft bg-bg-elev/20"
          : "border-border-soft/60 bg-bg-elev/10 opacity-70",
      )}
    >
      <div
        className="flex items-center gap-2.5 px-3 py-2 cursor-pointer hover:bg-bg-elev/30"
        onClick={onToggleExpand}
      >
        <input
          type="checkbox"
          checked={rule.enabled}
          onChange={(e) => {
            e.stopPropagation();
            onToggleEnable();
          }}
          onClick={(e) => e.stopPropagation()}
          disabled={busy}
          className="accent-accent w-3.5 h-3.5 flex-shrink-0"
        />

        <code className="font-mono text-[11px] text-text-muted flex-shrink-0">{rule.id}</code>

        <span
          className="text-[10px] font-mono font-bold uppercase tracking-wider px-1.5 py-0.5 rounded flex-shrink-0"
          style={{
            background: `color-mix(in srgb, ${actionMeta.color} 14%, transparent)`,
            color: actionMeta.color,
          }}
        >
          {actionMeta.label}
        </span>

        {effectiveAction && (
          <span className="text-[10px] font-mono text-amber-500 flex-shrink-0">
            ↳ {effectiveAction.toUpperCase()}
          </span>
        )}

        {rule.category && (
          <span className="text-[10px] font-mono text-text-muted px-1.5 py-0.5 rounded bg-bg-elev/40 flex-shrink-0">
            {rule.category}
          </span>
        )}

        <span className="text-[12px] text-text flex-1 truncate">
          {rule.description}
        </span>

        {rule.hits_7d > 0 && (
          <span
            className="text-[10.5px] font-mono px-1.5 py-0.5 rounded flex-shrink-0"
            style={{
              background: "color-mix(in srgb, rgb(var(--accent)) 10%, transparent)",
              color: "rgb(var(--accent))",
            }}
            title="过去 7 天命中次数"
          >
            {rule.hits_7d} hits
          </span>
        )}

        <ChevronRight
          className={cn(
            "w-3.5 h-3.5 text-text-muted flex-shrink-0 transition-transform",
            expanded && "rotate-90",
          )}
        />
      </div>

      {expanded && (
        <div className="px-3 pb-3 pt-1 border-t border-border-soft space-y-2">
          {rule.pattern_hint && (
            <DetailRow label="Pattern">
              <code className="font-mono text-[11px] text-text-dim break-all">
                {rule.pattern_hint}
              </code>
            </DetailRow>
          )}

          <DetailRow label="触发动作">
            <div className="flex items-center gap-1.5">
              <ActionChip
                label="默认"
                active={!effectiveAction}
                onClick={() => onSetAction(null)}
                tone="default"
              />
              <ActionChip
                label="Block"
                active={effectiveAction === "block"}
                onClick={() => onSetAction("block")}
                tone="block"
              />
              <ActionChip
                label="Warn"
                active={effectiveAction === "warn"}
                onClick={() => onSetAction("warn")}
                tone="warn"
              />
              <ActionChip
                label="Skip"
                active={effectiveAction === "skip"}
                onClick={() => onSetAction("skip")}
                tone="skip"
              />
            </div>
          </DetailRow>

          {rule.remediation && (
            <DetailRow label="修复建议">
              <span className="text-[11.5px] text-text-dim italic">{rule.remediation}</span>
            </DetailRow>
          )}

          <div className="flex items-center justify-end pt-1">
            <button
              onClick={onReset}
              className="flex items-center gap-1 text-[10.5px] px-2 py-1 rounded text-text-muted hover:text-text"
            >
              <RotateCcw className="w-3 h-3" />
              恢复默认
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-start gap-3">
      <span className="text-[10.5px] font-mono text-text-muted uppercase tracking-wider w-20 flex-shrink-0 pt-0.5">
        {label}
      </span>
      <div className="flex-1 min-w-0">{children}</div>
    </div>
  );
}

function ActionChip({
  label, active, onClick, tone,
}: {
  label: string; active: boolean; onClick: () => void; tone: "default" | "block" | "warn" | "skip";
}) {
  const color =
    tone === "block" ? "rgb(var(--critical))"
    : tone === "warn" ? "rgb(var(--high))"
    : tone === "skip" ? "rgb(var(--text-muted))"
    : "rgb(var(--accent))";
  return (
    <button
      onClick={onClick}
      className={cn(
        "text-[10.5px] font-mono px-2 py-0.5 rounded border transition-colors",
        active ? "" : "border-border-soft text-text-muted hover:text-text",
      )}
      style={
        active
          ? { borderColor: color, color, background: `color-mix(in srgb, ${color} 10%, transparent)` }
          : undefined
      }
    >
      {label}
    </button>
  );
}

function EmptyState() {
  return (
    <div className="text-center py-12">
      <div className="text-text-muted text-[12.5px]">当前过滤无规则</div>
      <div className="text-text-muted text-[11px] mt-1">
        调整搜索词或取消「仅显示已禁用」
      </div>
    </div>
  );
}

function ConfirmDisableDialog({
  rule, onCancel, onConfirm,
}: {
  rule: SecurityRuleRow; onCancel: () => void; onConfirm: () => void;
}) {
  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-md bg-bg rounded-xl shadow-2xl border border-border overflow-hidden">
        <header className="flex items-center gap-2.5 px-5 py-3.5 border-b border-border">
          <AlertTriangle className="w-4 h-4 text-critical" />
          <h3 className="text-[14px] font-semibold tracking-tight flex-1">
            禁用高危规则
          </h3>
          <button onClick={onCancel} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="px-5 py-4 space-y-3">
          <p className="text-[12.5px] text-text-dim leading-relaxed">
            你即将禁用一条默认 <strong className="text-critical">{ACTION_META[rule.default_action].label}</strong> 级别的规则：
          </p>
          <div className="rounded-md border border-critical/30 bg-critical/[0.04] px-3 py-2">
            <div className="flex items-center gap-2">
              <code className="font-mono text-[11.5px] text-text-muted">{rule.id}</code>
              {rule.category && (
                <span className="text-[10px] font-mono text-text-muted">{rule.category}</span>
              )}
            </div>
            <div className="text-[12px] mt-1">{rule.description}</div>
          </div>
          <p className="text-[11.5px] text-amber-500 leading-relaxed">
            ⚠ 禁用后该规则将不再拦截任何命中。如果你只是想降低强度，建议改为
            <strong> 触发动作 → Warn</strong> 而非完全禁用。
          </p>
        </div>

        <footer className="px-5 py-3 border-t border-border bg-bg-elev/30 flex items-center justify-end gap-2">
          <button
            onClick={onCancel}
            className="px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-md text-[12.5px] font-medium text-white bg-critical hover:bg-critical/90"
          >
            <Loader2 className="w-3 h-3 hidden" />
            确认禁用
          </button>
        </footer>
      </div>
    </div>
  );
}
