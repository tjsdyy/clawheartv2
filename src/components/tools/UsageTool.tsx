import { useState } from "react";
import { Wallet, TrendingUp, AlertCircle, Server, Cpu } from "lucide-react";
import { cn } from "@/lib/utils";
import { Sparkline } from "@/components/ui/Sparkline";
import {
  useUsageSummary,
  useUsageTrends,
  useUsageByProvider,
  useUsageByModel,
} from "@/hooks/useUsage";

const RANGES = [
  { id: 7, label: "近 7 天" },
  { id: 14, label: "近 14 天" },
  { id: 30, label: "近 30 天" },
];

export function UsageTool() {
  const [days, setDays] = useState(14);
  const { data: summary } = useUsageSummary();
  const { data: trends = [] } = useUsageTrends(days);
  const { data: byProvider = [] } = useUsageByProvider(days);
  const { data: byModel = [] } = useUsageByModel(days);

  const costSeries = trends.map((t) => t.cost_usd);
  const inSeries = trends.map((t) => t.input_tokens);
  const outSeries = trends.map((t) => t.output_tokens);

  const totalCost = trends.reduce((s, t) => s + t.cost_usd, 0);
  const totalIn = trends.reduce((s, t) => s + t.input_tokens, 0);
  const totalOut = trends.reduce((s, t) => s + t.output_tokens, 0);

  return (
    <div className="mx-auto py-8 px-12" style={{ maxWidth: 900 }}>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-[22px] font-semibold tracking-tight mb-1">Token 用量统计</h2>
          <p className="text-[13px] text-text-dim">
            从 token_usage 表实时聚合 · 按 provider / model 拆分 · sparkline 趋势
          </p>
        </div>
        <div className="flex gap-1 bg-bg-elev2 rounded-md p-1">
          {RANGES.map((r) => (
            <button
              key={r.id}
              onClick={() => setDays(r.id)}
              className={cn(
                "px-3 py-1 text-[12px] rounded transition-colors",
                days === r.id ? "bg-bg text-text shadow-sm" : "text-text-dim hover:text-text",
              )}
            >
              {r.label}
            </button>
          ))}
        </div>
      </div>

      {/* 今日总览 4 stat cards */}
      <div className="grid grid-cols-4 gap-3 mb-6">
        <StatCard
          icon={<Wallet className="w-4 h-4" />}
          label="今日成本"
          value={`$${summary?.cost_usd.toFixed(4) ?? "0.0000"}`}
          color="green"
        />
        <StatCard
          icon={<TrendingUp className="w-4 h-4" />}
          label="今日请求"
          value={summary?.request_count.toLocaleString() ?? "0"}
          color="blue"
        />
        <StatCard
          icon={<Cpu className="w-4 h-4" />}
          label="今日 Token"
          value={`${formatTokens((summary?.input_tokens ?? 0) + (summary?.output_tokens ?? 0))}`}
          color="purple"
          hint={`↓${formatTokens(summary?.input_tokens ?? 0)} ↑${formatTokens(summary?.output_tokens ?? 0)}`}
        />
        <StatCard
          icon={<AlertCircle className="w-4 h-4" />}
          label="今日拦截"
          value={summary?.blocked_count.toString() ?? "0"}
          color="red"
        />
      </div>

      {/* 趋势 sparkline cards */}
      <div className="grid grid-cols-3 gap-3 mb-6">
        <TrendCard
          label="成本趋势"
          value={`$${totalCost.toFixed(2)}`}
          hint={`${days} 天累计`}
          data={costSeries}
          color="rgb(var(--accent))"
        />
        <TrendCard
          label="输入 Token"
          value={formatTokens(totalIn)}
          hint={`${days} 天累计`}
          data={inSeries}
          color="rgb(var(--tool-monitor))"
        />
        <TrendCard
          label="输出 Token"
          value={formatTokens(totalOut)}
          hint={`${days} 天累计`}
          data={outSeries}
          color="rgb(var(--tool-skills))"
        />
      </div>

      {/* 按 provider 汇总 */}
      <Section title="按 Provider 汇总" icon={<Server className="w-3.5 h-3.5" />}>
        {byProvider.length === 0 ? (
          <Empty hint={`${days} 天内无用量数据 · 等代理引擎 W5 启用后会自动产生`} />
        ) : (
          <table className="w-full text-[12.5px]">
            <thead>
              <tr className="text-text-muted text-[10px] uppercase tracking-wider font-mono">
                <Th>Provider</Th>
                <Th right>请求</Th>
                <Th right>输入</Th>
                <Th right>输出</Th>
                <Th right>成本</Th>
              </tr>
            </thead>
            <tbody>
              {byProvider.map((r) => (
                <tr key={r.provider} className="border-t border-border-soft">
                  <Td>{r.provider}</Td>
                  <Td right mono>{r.request_count}</Td>
                  <Td right mono>{formatTokens(r.input_tokens)}</Td>
                  <Td right mono>{formatTokens(r.output_tokens)}</Td>
                  <Td right mono strong>${r.cost_usd.toFixed(4)}</Td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>

      {/* 按 model 汇总 */}
      <Section title="按 Model 汇总" icon={<Cpu className="w-3.5 h-3.5" />} className="mt-4">
        {byModel.length === 0 ? (
          <Empty hint="无数据" />
        ) : (
          <table className="w-full text-[12.5px]">
            <thead>
              <tr className="text-text-muted text-[10px] uppercase tracking-wider font-mono">
                <Th>Provider · Model</Th>
                <Th right>请求</Th>
                <Th right>输入 / 输出</Th>
                <Th right>成本</Th>
              </tr>
            </thead>
            <tbody>
              {byModel.map((r) => (
                <tr key={`${r.provider}-${r.model}`} className="border-t border-border-soft">
                  <Td>
                    <span className="text-text-muted">{r.provider}</span>
                    <span className="opacity-40 mx-1">·</span>
                    <span className="font-mono">{r.model}</span>
                  </Td>
                  <Td right mono>{r.request_count}</Td>
                  <Td right mono dim>
                    ↓{formatTokens(r.input_tokens)} ↑{formatTokens(r.output_tokens)}
                  </Td>
                  <Td right mono strong>${r.cost_usd.toFixed(4)}</Td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>
    </div>
  );
}

function StatCard({
  icon, label, value, color, hint,
}: { icon: React.ReactNode; label: string; value: string; color: string; hint?: string }) {
  const tonePalette: Record<string, string> = {
    green: "rgb(var(--accent))",
    blue: "rgb(var(--tool-monitor))",
    purple: "rgb(var(--tool-skills))",
    red: "rgb(var(--critical))",
  };
  const tone = tonePalette[color] ?? "rgb(var(--accent))";
  return (
    <div className="surface p-3.5">
      <div className="flex items-center gap-2 mb-2">
        <span
          className="w-6 h-6 rounded-md flex items-center justify-center"
          style={{ background: `color-mix(in srgb, ${tone} 14%, transparent)`, color: tone }}
        >
          {icon}
        </span>
        <span className="text-[11px] text-text-muted font-mono uppercase tracking-wider">{label}</span>
      </div>
      <div className="text-[20px] font-semibold leading-none tracking-tight">{value}</div>
      {hint && <div className="text-[10.5px] text-text-muted font-mono mt-1.5">{hint}</div>}
    </div>
  );
}

function TrendCard({
  label, value, hint, data, color,
}: { label: string; value: string; hint: string; data: number[]; color: string }) {
  return (
    <div className="surface p-3.5">
      <div className="flex items-center justify-between mb-1">
        <span className="text-[11px] text-text-muted font-mono uppercase tracking-wider">{label}</span>
        <span className="text-[10px] text-text-muted">{hint}</span>
      </div>
      <div className="text-[18px] font-semibold leading-none tracking-tight mb-2">{value}</div>
      <Sparkline data={data} width={180} height={36} color={color} />
    </div>
  );
}

function Section({
  title, icon, children, className,
}: { title: string; icon?: React.ReactNode; children: React.ReactNode; className?: string }) {
  return (
    <div className={cn("surface", className)}>
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border-soft text-[12.5px] font-semibold">
        {icon}
        <span>{title}</span>
      </div>
      <div className="p-3">{children}</div>
    </div>
  );
}

function Empty({ hint }: { hint: string }) {
  return <div className="py-8 text-center text-text-muted text-[12px] font-mono">{hint}</div>;
}

function Th({ children, right }: { children: React.ReactNode; right?: boolean }) {
  return <th className={cn("px-3 py-2 font-bold", right && "text-right")}>{children}</th>;
}

function Td({
  children, right, dim, mono, strong,
}: { children: React.ReactNode; right?: boolean; dim?: boolean; mono?: boolean; strong?: boolean }) {
  return (
    <td
      className={cn(
        "px-3 py-2",
        right && "text-right",
        dim && "text-text-dim",
        mono && "font-mono",
        strong && "font-semibold",
      )}
    >
      {children}
    </td>
  );
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}
