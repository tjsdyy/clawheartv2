import { Loader2, RefreshCcw } from "lucide-react";
import { cn } from "@/lib/utils";
import type { DiscoveredSkill, SsotConfig } from "@/hooks/useSkills";
import type { LocalSkillScanReport } from "@/hooks/useSkills";

export type StatusFilter = null | "managed" | "unmanaged" | "orphan";
export type ScanFilter = null | "unscanned" | "safe" | "warn" | "critical";

export interface FilterState {
  agent: string | null;       // .claude / .openeva / ... ；null = 全部
  status: StatusFilter;       // 集中库状态
  scan: ScanFilter;           // 鉴定结果
}

interface Props {
  skills: DiscoveredSkill[];
  reports: Record<string, LocalSkillScanReport>;
  filter: FilterState;
  onFilterChange: (next: FilterState) => void;
  ssotConfig?: SsotConfig;
  onRefetch: () => void;
  fetching: boolean;
}

export function SkillsFilterSidebar({
  skills,
  reports,
  filter,
  onFilterChange,
  ssotConfig,
  onRefetch,
  fetching,
}: Props) {
  const counts = countAll(skills, reports);
  const agentEntries = countByAgent(skills);

  return (
    <aside className="border-r border-border-soft p-3 overflow-auto text-[12px]">
      {/* ─── 概览 ─── */}
      <SectionLabel>按管理状态</SectionLabel>
      <FilterButton
        label="全部"
        count={skills.length}
        active={!filter.status && !filter.agent && !filter.scan}
        onClick={() =>
          onFilterChange({ agent: null, status: null, scan: null })
        }
      />
      <FilterButton
        label="未管理"
        hint="散落在 Agent 目录的真实文件"
        count={counts.unmanaged}
        accent="rgb(245 158 11)"
        active={filter.status === "unmanaged"}
        onClick={() =>
          onFilterChange({
            ...filter,
            status: filter.status === "unmanaged" ? null : "unmanaged",
          })
        }
      />
      <FilterButton
        label="已集中"
        hint="主副本在集中库 ~/.agents/skills/ 中"
        count={counts.managed}
        accent="rgb(var(--accent))"
        active={filter.status === "managed"}
        onClick={() =>
          onFilterChange({
            ...filter,
            status: filter.status === "managed" ? null : "managed",
          })
        }
      />
      <FilterButton
        label="未启用 (Orphan)"
        hint="集中库中存在但所有 Agent 都未启用"
        count={counts.orphan}
        active={filter.status === "orphan"}
        onClick={() =>
          onFilterChange({
            ...filter,
            status: filter.status === "orphan" ? null : "orphan",
          })
        }
      />

      {/* ─── Agent ─── */}
      <SectionLabel className="mt-4">按 Agent</SectionLabel>
      {agentEntries.map(({ agent, count }) => (
        <FilterButton
          key={agent}
          label={`.${agent}`}
          mono
          count={count}
          active={filter.agent === agent}
          onClick={() =>
            onFilterChange({
              ...filter,
              agent: filter.agent === agent ? null : agent,
            })
          }
        />
      ))}

      {/* ─── 鉴定 ─── */}
      <SectionLabel className="mt-4">按安全等级</SectionLabel>
      <FilterButton
        label="未扫描"
        count={counts.unscanned}
        active={filter.scan === "unscanned"}
        onClick={() =>
          onFilterChange({
            ...filter,
            scan: filter.scan === "unscanned" ? null : "unscanned",
          })
        }
      />
      <FilterButton
        label="✓ 安全"
        count={counts.safe}
        accent="rgb(var(--accent))"
        active={filter.scan === "safe"}
        onClick={() =>
          onFilterChange({
            ...filter,
            scan: filter.scan === "safe" ? null : "safe",
          })
        }
      />
      <FilterButton
        label="⚠ 警告"
        count={counts.warn}
        accent="rgb(245 158 11)"
        active={filter.scan === "warn"}
        onClick={() =>
          onFilterChange({
            ...filter,
            scan: filter.scan === "warn" ? null : "warn",
          })
        }
      />
      <FilterButton
        label="✗ 危险"
        count={counts.critical}
        accent="rgb(var(--critical))"
        active={filter.scan === "critical"}
        onClick={() =>
          onFilterChange({
            ...filter,
            scan: filter.scan === "critical" ? null : "critical",
          })
        }
      />

      {/* ─── 操作 + 集中库信息 ─── */}
      <button
        onClick={onRefetch}
        disabled={fetching}
        className="mt-4 w-full flex items-center justify-center gap-1.5 text-[11.5px] text-text-muted hover:text-text px-2 py-1.5 border border-border-soft rounded-md disabled:opacity-50"
      >
        {fetching ? (
          <Loader2 className="w-3 h-3 animate-spin" />
        ) : (
          <RefreshCcw className="w-3 h-3" />
        )}
        重新扫描
      </button>

      {ssotConfig && (
        <div className="mt-3 rounded-md border border-border-soft px-2 py-1.5 text-[10.5px] leading-relaxed">
          <div className="text-text-muted uppercase tracking-wider font-bold font-mono mb-0.5">
            集中库
          </div>
          <code className="text-text-dim break-all">{ssotConfig.path}</code>
          <div className="font-mono text-text-muted mt-0.5">
            {ssotConfig.exists
              ? `${ssotConfig.total_skills} 项 · ${formatBytes(ssotConfig.total_bytes)}`
              : "尚未创建"}
          </div>
        </div>
      )}

      <p className="mt-3 text-[10.5px] text-text-muted leading-relaxed px-1">
        扫描 <code className="text-text-dim">~/.&lt;agent&gt;/skills/</code>
        ；任何带 skills 子目录的 Agent 都会自动捕获
      </p>
    </aside>
  );
}

// ──────────────────────────────────────────────────────────────────
// 子组件
// ──────────────────────────────────────────────────────────────────
function SectionLabel({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "text-[10px] text-text-muted uppercase tracking-wider font-bold font-mono px-2 mb-1.5",
        className,
      )}
    >
      {children}
    </div>
  );
}

function FilterButton({
  label,
  hint,
  count,
  accent,
  active,
  mono,
  onClick,
}: {
  label: string;
  hint?: string;
  count: number;
  accent?: string;
  active: boolean;
  mono?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full text-left px-2.5 py-1.5 rounded-md flex items-center justify-between mb-0.5 transition-colors",
        active ? "bg-bg-elev/70 text-text" : "text-text-dim hover:text-text",
      )}
      title={hint}
    >
      <span className={cn("text-[12.5px]", mono && "font-mono")} style={accent ? { color: active ? accent : undefined } : undefined}>
        {label}
      </span>
      <span
        className="text-[10.5px] font-mono"
        style={{ color: accent && count > 0 ? accent : "rgb(var(--text-muted))" }}
      >
        {count}
      </span>
    </button>
  );
}

// ──────────────────────────────────────────────────────────────────
// 计数
// ──────────────────────────────────────────────────────────────────
function countAll(
  skills: DiscoveredSkill[],
  reports: Record<string, LocalSkillScanReport>,
) {
  let managed = 0;
  let unmanaged = 0;
  let orphan = 0;
  let unscanned = 0;
  let safe = 0;
  let warn = 0;
  let critical = 0;

  for (const s of skills) {
    if (s.in_ssot) {
      managed += 1;
      const enabledCount = s.bindings.filter(
        (b) => b.binding.kind === "symlink" && b.binding.points_to_ssot,
      ).length;
      if (enabledCount === 0) orphan += 1;
    } else {
      unmanaged += 1;
    }

    const r = reports[s.id];
    if (!r) {
      unscanned += 1;
    } else if (r.blocked) {
      critical += 1;
    } else if (r.score >= 80) {
      safe += 1;
    } else if (r.score >= 50) {
      warn += 1;
    } else {
      critical += 1;
    }
  }

  return { managed, unmanaged, orphan, unscanned, safe, warn, critical };
}

function countByAgent(skills: DiscoveredSkill[]) {
  const m = new Map<string, number>();
  for (const s of skills) {
    // 优先按 binding 计算；没有 binding 时用 source_agent
    const agents = s.bindings.length > 0
      ? Array.from(new Set(s.bindings.map((b) => b.agent_name)))
      : [s.source_agent];
    for (const a of agents) {
      m.set(a, (m.get(a) ?? 0) + 1);
    }
  }
  return Array.from(m.entries())
    .map(([agent, count]) => ({ agent, count }))
    .sort((a, b) => b.count - a.count);
}

function formatBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(2)} MB`;
  if (n >= 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${n} B`;
}
