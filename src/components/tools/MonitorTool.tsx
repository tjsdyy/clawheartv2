import { useState } from "react";
import { CheckCircle2, XCircle, ClipboardCopy, Activity, ShieldX, Coins } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { useRequestLogs, type RequestLogItem } from "@/hooks/useRequestLogs";
import { useRecentEvents } from "@/hooks/useStatus";
import { useTokenUsage, type TokenUsageDay } from "@/hooks/useBudget";
import { useToolLayoutTab } from "./ToolLayout";
import { BudgetTool } from "./BudgetTool";

/**
 * 监控工具入口 — 由 ToolLayout 的 4 个 tab 切换 4 个子视图。
 * 顺序与 App.tsx 注册的 tabs 保持一致：
 *   0 实时流 · 1 拦截记录 · 2 Token 用量 · 3 预算
 */
export function MonitorTool() {
  const tab = useToolLayoutTab();
  switch (tab) {
    case 0:
      return <RealtimeStreamView />;
    case 1:
      return <BlockedEventsView />;
    case 2:
      return <TokenUsageView />;
    case 3:
      return <BudgetTool />;
    default:
      return <RealtimeStreamView />;
  }
}

// ──────────────────────────────────────────────────────────────────
// 视图 0：实时流（原 MonitorTool 三栏布局）
// ──────────────────────────────────────────────────────────────────
function RealtimeStreamView() {
  const { data: rows = [], isLoading } = useRequestLogs(100);
  const { data: events = [] } = useRecentEvents();
  const [selected, setSelected] = useState<number | null>(null);

  const selectedEvent = selected !== null ? events[selected] ?? null : null;

  return (
    <div className="grid h-full" style={{ gridTemplateColumns: "220px 1fr 320px" }}>
      {/* Filter */}
      <aside className="border-r border-border-soft p-4 overflow-auto">
        <FilterGroup title="严重等级">
          <FilterItem label="Critical" color="critical" defaultChecked />
          <FilterItem label="High" color="high" defaultChecked />
          <FilterItem label="Medium" color="medium" defaultChecked />
          <FilterItem label="Low" color="low" />
        </FilterGroup>
        <FilterGroup title="事件类型">
          <FilterItem label="MCP" defaultChecked />
          <FilterItem label="凭据" defaultChecked />
          <FilterItem label="危险指令" defaultChecked />
          <FilterItem label="预算" defaultChecked />
          <FilterItem label="漂移" />
        </FilterGroup>
      </aside>

      {/* Stream */}
      <section className="border-r border-border-soft overflow-auto font-mono text-[12px]">
        <div
          className="grid gap-3 px-4 py-2.5 border-b border-border-soft bg-bg/75 backdrop-blur-sm sticky top-0 z-10 text-[9.5px] uppercase tracking-wider font-bold text-text-muted"
          style={{ gridTemplateColumns: "88px 1fr 1fr 50px" }}
        >
          <span>TIME</span>
          <span>AGENT</span>
          <span>ENDPOINT</span>
          <span className="text-right">RES</span>
        </div>

        {isLoading && (
          <div className="px-6 py-12 text-center text-text-muted font-sans">加载中…</div>
        )}

        {!isLoading && rows.length === 0 && <EmptyStream />}

        {rows.map((r, i) => (
          <button
            key={r.id}
            onClick={() => setSelected(i)}
            className={cn(
              "grid gap-3 px-4 py-2.5 border-b border-border-soft items-center w-full text-left transition-colors",
              "hover:bg-bg-elev",
              r.blocked && "bg-critical/5",
              selected === i && "bg-accent/10 border-l-2 border-l-accent",
            )}
            style={{ gridTemplateColumns: "88px 1fr 1fr 50px" }}
          >
            <span className="text-text-muted">{r.timestamp}</span>
            <span className="text-text">{r.agent_id ?? "—"}</span>
            <span className="text-text-dim">{r.endpoint}</span>
            <span
              className={cn(
                "text-right font-bold tracking-wide text-[11px]",
                r.status_code === 200 && "text-accent",
                r.blocked && "text-critical",
                r.status_code === 429 && "text-high",
              )}
            >
              {r.blocked ? "BLK" : r.status_code}
            </span>
          </button>
        ))}
      </section>

      {/* Detail */}
      <aside className="border-l border-border-soft p-5 overflow-auto">
        {!selectedEvent && (
          <div className="text-text-muted text-[12px] font-mono text-center py-8">
            {events.length === 0
              ? "暂无拦截事件\n（启用代理 + 让 Agent 跑后填充）"
              : "选择左侧请求查看详情"}
          </div>
        )}

        {selectedEvent && (
          <>
            <div className="font-mono text-[11px] text-text-muted mb-2.5 tracking-wide">
              {selectedEvent.timestamp} · {selectedEvent.agent}
            </div>

            <div
              className="rounded-xl p-3.5 mb-4 bg-bg-elev2 border-l-[3px]"
              style={{ borderLeftColor: `rgb(var(--${selectedEvent.severity}))` }}
            >
              <div className="flex items-center gap-2 mb-1.5 flex-wrap">
                <span
                  className="px-2 py-0.5 rounded text-[9.5px] font-bold tracking-wider font-mono text-white"
                  style={{ background: `rgb(var(--${selectedEvent.severity}))` }}
                >
                  {selectedEvent.severity.toUpperCase()}
                </span>
                <span className="font-mono text-[10.5px] text-text-muted px-1.5 py-0.5 rounded bg-bg">
                  {selectedEvent.event_type}
                </span>
              </div>
              <div className="font-semibold text-sm mt-1">{selectedEvent.label}</div>
            </div>

            <div className="flex flex-col gap-1.5 mt-4">
              <ActionBtn
                icon={<CheckCircle2 className="w-3.5 h-3.5" />}
                onClick={() => toast.info("允许序列规则在 W11 实现")}
              >
                允许该序列
              </ActionBtn>
              <ActionBtn
                icon={<XCircle className="w-3.5 h-3.5" />}
                danger
                onClick={() => toast.info("永久阻止规则在 W11 实现")}
              >
                永久阻止
              </ActionBtn>
              <ActionBtn
                icon={<ClipboardCopy className="w-3.5 h-3.5" />}
                onClick={() => {
                  navigator.clipboard.writeText(
                    `${selectedEvent.timestamp} ${selectedEvent.event_type} ${selectedEvent.label}`,
                  );
                  toast.success("已脱敏复制事件 ID");
                }}
              >
                脱敏复制 ID
              </ActionBtn>
            </div>
          </>
        )}
      </aside>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 视图 1：拦截记录（blocked=true）
// ──────────────────────────────────────────────────────────────────
function BlockedEventsView() {
  const { data: rows = [], isLoading } = useRequestLogs(300);
  const blocked = rows.filter((r) => r.blocked);

  return (
    <div className="mx-auto py-6 px-8" style={{ maxWidth: 1100 }}>
      <div className="mb-5 flex items-center justify-between">
        <div>
          <h2 className="text-[18px] font-semibold tracking-tight">拦截记录</h2>
          <p className="text-[12px] text-text-dim mt-0.5">
            被 ClawHeart 8 层防御拦截的请求 · 共 {blocked.length} 条
          </p>
        </div>
        <span className="text-[11px] font-mono text-text-muted">显示最近 300 条扫描结果中的 BLK</span>
      </div>

      {isLoading && (
        <div className="text-center py-12 text-text-muted text-[13px]">加载中…</div>
      )}

      {!isLoading && blocked.length === 0 && (
        <div className="rounded-xl border border-border-soft py-16 px-8 text-center">
          <div className="inline-flex w-12 h-12 rounded-xl items-center justify-center mb-3"
            style={{
              background: "color-mix(in srgb, rgb(var(--critical)) 12%, transparent)",
              color: "rgb(var(--critical))",
            }}
          >
            <ShieldX className="w-6 h-6" />
          </div>
          <h3 className="text-[14px] font-semibold mb-1.5">暂无拦截事件</h3>
          <p className="text-[12px] text-text-dim leading-relaxed max-w-sm mx-auto">
            当 8 层防御检测到危险指令、凭据泄漏、超额预算等风险时，被阻止的请求会出现在这里。
          </p>
        </div>
      )}

      {blocked.length > 0 && (
        <div className="rounded-xl border border-border-soft overflow-hidden font-mono text-[11.5px]">
          <div
            className="grid gap-3 px-4 py-2.5 bg-bg/60 backdrop-blur-sm border-b border-border-soft text-[10px] uppercase tracking-wider font-bold text-text-muted"
            style={{ gridTemplateColumns: "100px 1fr 1fr 60px 80px" }}
          >
            <span>TIME</span>
            <span>AGENT</span>
            <span>ENDPOINT</span>
            <span className="text-right">RES</span>
            <span className="text-right">LAT</span>
          </div>
          {blocked.map((r) => (
            <BlockedRow key={r.id} r={r} />
          ))}
        </div>
      )}
    </div>
  );
}

function BlockedRow({ r }: { r: RequestLogItem }) {
  return (
    <div
      className="grid gap-3 px-4 py-2 border-b border-border-soft hover:bg-bg-elev/40 items-center bg-critical/[0.03]"
      style={{ gridTemplateColumns: "100px 1fr 1fr 60px 80px" }}
    >
      <span className="text-text-muted">{r.timestamp}</span>
      <span className="text-text">{r.agent_id ?? "—"}</span>
      <span className="text-text-dim truncate">{r.endpoint}</span>
      <span className="text-right font-bold text-critical">BLK</span>
      <span className="text-right text-text-muted">{r.latency_ms}ms</span>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 视图 2：Token 用量
// ──────────────────────────────────────────────────────────────────
function TokenUsageView() {
  const [days, setDays] = useState<7 | 14 | 30>(7);
  const { data: usage = [], isLoading } = useTokenUsage(days);

  // 按时间升序便于图表 left→right
  const sorted = [...usage].sort((a, b) => a.date.localeCompare(b.date));
  const totalIn = sorted.reduce((s, d) => s + d.input_tokens, 0);
  const totalOut = sorted.reduce((s, d) => s + d.output_tokens, 0);
  const totalCost = sorted.reduce((s, d) => s + d.cost_usd, 0);
  const maxTokens = Math.max(1, ...sorted.map((d) => d.input_tokens + d.output_tokens));

  return (
    <div className="mx-auto py-6 px-8" style={{ maxWidth: 1100 }}>
      <div className="mb-5 flex items-center justify-between">
        <div>
          <h2 className="text-[18px] font-semibold tracking-tight">Token 用量</h2>
          <p className="text-[12px] text-text-dim mt-0.5">
            来自 token_usage 表 · 按天聚合 · 含成本估算
          </p>
        </div>
        <div className="flex items-center gap-1 text-[11.5px]">
          {([7, 14, 30] as const).map((d) => (
            <button
              key={d}
              onClick={() => setDays(d)}
              className={cn(
                "px-2.5 py-1 rounded-md font-medium transition-colors",
                days === d
                  ? "bg-bg-elev/70 text-text"
                  : "text-text-muted hover:text-text",
              )}
            >
              {d} 天
            </button>
          ))}
        </div>
      </div>

      {/* 概览卡 */}
      <div className="grid grid-cols-3 gap-3 mb-5">
        <StatCard
          label="输入 Token"
          value={formatNum(totalIn)}
          accent="rgb(6 182 212)"
        />
        <StatCard
          label="输出 Token"
          value={formatNum(totalOut)}
          accent="rgb(168 85 247)"
        />
        <StatCard
          label="累计成本"
          value={`$${totalCost.toFixed(2)}`}
          accent="rgb(var(--accent))"
        />
      </div>

      {/* 趋势图（CSS 柱状） */}
      <div className="rounded-xl border border-border-soft p-5 mb-5">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-[12.5px] font-semibold">每日 Token 趋势</h3>
          <div className="flex items-center gap-3 text-[10.5px] text-text-muted font-mono">
            <Legend color="rgb(6 182 212)" label="input" />
            <Legend color="rgb(168 85 247)" label="output" />
          </div>
        </div>

        {isLoading ? (
          <div className="text-center py-10 text-text-muted text-[12px]">加载中…</div>
        ) : sorted.length === 0 ? (
          <EmptyUsage />
        ) : (
          <div className="flex items-end gap-2 h-40">
            {sorted.map((d) => {
              const total = d.input_tokens + d.output_tokens;
              const inH = (d.input_tokens / maxTokens) * 100;
              const outH = (d.output_tokens / maxTokens) * 100;
              return (
                <div
                  key={d.date}
                  className="flex-1 flex flex-col items-center gap-1 group"
                  title={`${d.date}\n输入: ${formatNum(d.input_tokens)}\n输出: ${formatNum(d.output_tokens)}\n成本: $${d.cost_usd.toFixed(2)}`}
                >
                  <div className="text-[9px] font-mono text-text-muted opacity-0 group-hover:opacity-100 transition-opacity">
                    {formatNum(total)}
                  </div>
                  <div className="w-full flex flex-col-reverse" style={{ height: "100%" }}>
                    <div
                      className="w-full rounded-b transition-all"
                      style={{
                        height: `${inH}%`,
                        background: "rgb(6 182 212)",
                        opacity: 0.85,
                      }}
                    />
                    <div
                      className="w-full rounded-t transition-all"
                      style={{
                        height: `${outH}%`,
                        background: "rgb(168 85 247)",
                        opacity: 0.85,
                      }}
                    />
                  </div>
                  <div className="text-[9.5px] font-mono text-text-muted tabular-nums">
                    {d.date.slice(5)}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* 明细表 */}
      <div className="rounded-xl border border-border-soft overflow-hidden">
        <div
          className="grid gap-3 px-4 py-2.5 bg-bg/60 backdrop-blur-sm border-b border-border-soft text-[10px] uppercase tracking-wider font-bold text-text-muted font-mono"
          style={{ gridTemplateColumns: "110px 1fr 1fr 1fr 110px" }}
        >
          <span>DATE</span>
          <span className="text-right">INPUT</span>
          <span className="text-right">OUTPUT</span>
          <span className="text-right">TOTAL</span>
          <span className="text-right">COST</span>
        </div>
        {[...sorted].reverse().map((d) => (
          <UsageRow key={d.date} d={d} />
        ))}
        {!isLoading && sorted.length === 0 && (
          <div className="py-8 text-center text-[11.5px] text-text-muted">暂无用量数据</div>
        )}
      </div>
    </div>
  );
}

function UsageRow({ d }: { d: TokenUsageDay }) {
  return (
    <div
      className="grid gap-3 px-4 py-2 border-b border-border-soft hover:bg-bg-elev/40 items-center font-mono text-[11.5px]"
      style={{ gridTemplateColumns: "110px 1fr 1fr 1fr 110px" }}
    >
      <span className="text-text">{d.date}</span>
      <span className="text-right text-text-dim">{formatNum(d.input_tokens)}</span>
      <span className="text-right text-text-dim">{formatNum(d.output_tokens)}</span>
      <span className="text-right text-text">{formatNum(d.input_tokens + d.output_tokens)}</span>
      <span className="text-right text-accent font-semibold">${d.cost_usd.toFixed(2)}</span>
    </div>
  );
}

function StatCard({ label, value, accent }: { label: string; value: string; accent: string }) {
  return (
    <div className="rounded-xl border border-border-soft p-4 relative overflow-hidden">
      <div
        className="absolute top-0 left-0 w-1 h-full"
        style={{ background: accent }}
      />
      <div className="text-[10.5px] text-text-muted uppercase tracking-wider font-mono mb-1">
        {label}
      </div>
      <div className="text-[20px] font-semibold tracking-tight tabular-nums">
        {value}
      </div>
    </div>
  );
}

function Legend({ color, label }: { color: string; label: string }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className="w-2 h-2 rounded-sm" style={{ background: color }} />
      <span>{label}</span>
    </span>
  );
}

function EmptyUsage() {
  return (
    <div className="text-center py-10">
      <div
        className="inline-flex w-12 h-12 rounded-xl items-center justify-center mb-3"
        style={{
          background: "color-mix(in srgb, rgb(var(--accent)) 12%, transparent)",
          color: "rgb(var(--accent))",
        }}
      >
        <Coins className="w-6 h-6" />
      </div>
      <h3 className="text-[13px] font-semibold mb-1">暂无 Token 用量</h3>
      <p className="text-[11.5px] text-text-dim leading-relaxed max-w-xs mx-auto">
        Agent 通过 ClawHeart 调用 LLM 后，token_usage 表会自动写入并在此聚合
      </p>
    </div>
  );
}

function formatNum(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

// ──────────────────────────────────────────────────────────────────
// 共用子组件（实时流栏内）
// ──────────────────────────────────────────────────────────────────
function EmptyStream() {
  return (
    <div className="flex flex-col items-center justify-center py-20 px-8 text-center font-sans">
      <div
        className="w-12 h-12 rounded-xl flex items-center justify-center mb-4"
        style={{
          background: "color-mix(in srgb, rgb(var(--tool-monitor)) 12%, transparent)",
          color: "rgb(var(--tool-monitor))",
        }}
      >
        <Activity className="w-6 h-6" />
      </div>
      <h3 className="text-[14px] font-semibold mb-2">还没有流量记录</h3>
      <p className="text-[12px] text-text-dim leading-relaxed max-w-xs">
        当 AI Agent 通过 ClawHeart 代理（127.0.0.1:19111）发起请求后，
        实时流量会显示在这里。
      </p>
    </div>
  );
}

function FilterGroup({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="mb-5">
      <h4 className="text-[10px] tracking-wider uppercase text-text-muted mb-2 font-bold font-mono">{title}</h4>
      <div className="space-y-0.5">{children}</div>
    </div>
  );
}

function FilterItem({ label, color, defaultChecked }: { label: string; color?: string; defaultChecked?: boolean }) {
  return (
    <label className="flex items-center gap-2 px-2 py-1.5 rounded-md text-text-dim hover:bg-bg-elev2 hover:text-text cursor-pointer text-[12.5px]">
      <input type="checkbox" defaultChecked={defaultChecked} className="accent-accent" />
      {color && (
        <span
          className="severity-dot"
          style={{ background: `rgb(var(--${color}))` }}
        />
      )}
      <span>{label}</span>
    </label>
  );
}

function ActionBtn({
  icon,
  children,
  danger,
  onClick,
}: {
  icon: React.ReactNode;
  children: React.ReactNode;
  danger?: boolean;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "px-3 py-2 rounded-lg text-[12.5px] bg-bg border border-border text-text",
        "flex items-center gap-2 text-left transition-colors",
        danger
          ? "hover:!border-critical hover:!text-critical hover:!bg-critical/5"
          : "hover:border-accent hover:text-accent",
      )}
    >
      {icon}
      {children}
    </button>
  );
}
