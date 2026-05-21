import {
  ChevronRight, FileText, Loader2, ShieldAlert, ShieldCheck, ShieldX,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { AgentToggleGroup, MoveToSsotInline } from "./AgentToggleGroup";
import {
  isBindingEnabled,
  type DiscoveredSkill,
  type LocalSkillScanReport,
} from "@/hooks/useSkills";

interface Props {
  skill: DiscoveredSkill;
  checked: boolean;
  onToggle: (v: boolean) => void;
  report?: LocalSkillScanReport;
  scanning: boolean;
  onScan: () => void;
  onShowDetail: () => void;
}

export function SkillRow({
  skill,
  checked,
  onToggle,
  report,
  scanning,
  onScan,
  onShowDetail,
}: Props) {
  return (
    <label
      className={cn(
        "flex items-start gap-3 p-3 rounded-lg border transition-colors cursor-pointer",
        checked
          ? "border-accent/50 bg-accent/5"
          : "border-border-soft hover:border-text-muted bg-bg-elev/20",
      )}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onToggle(e.target.checked)}
        className="mt-1 accent-accent w-3.5 h-3.5"
      />

      <div
        className="w-9 h-9 rounded-lg bg-bg-elev2 border border-border-soft flex items-center justify-center flex-shrink-0"
        style={{ color: "rgb(var(--accent))" }}
      >
        <FileText className="w-4 h-4" />
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <h4 className="font-mono font-semibold text-[13px] truncate">{skill.name}</h4>
          {skill.version && (
            <span className="text-[10px] font-mono text-text-muted px-1.5 py-0.5 rounded bg-bg-elev/50">
              v{skill.version}
            </span>
          )}
          {skill.in_ssot ? (
            <span
              className="text-[10px] font-mono px-1.5 py-0.5 rounded flex items-center gap-1"
              style={{
                background: "color-mix(in srgb, rgb(var(--accent)) 14%, transparent)",
                color: "rgb(var(--accent))",
              }}
              title="主副本在集中库 ~/.agents/skills/ 中 · 各 Agent 通过 symlink 共享"
            >
              ● 集中库
            </span>
          ) : (
            <span
              className="text-[10px] font-mono px-1.5 py-0.5 rounded text-amber-500 bg-amber-500/10"
              title="散落在 Agent 目录的真实文件 · 未纳入统一管理"
            >
              ⚠ 散落
            </span>
          )}
          {!skill.has_skill_md && (
            <span className="text-[10px] font-mono text-amber-500" title="未发现 SKILL.md">
              无 SKILL.md
            </span>
          )}
        </div>
        <p className="text-[11.5px] text-text-dim leading-snug truncate mb-1.5">
          {skill.description ?? <span className="text-text-muted">（无描述）</span>}
        </p>

        <AgentToggleGroup
          skillId={skill.id}
          inSsot={skill.in_ssot}
          bindings={skill.bindings}
        />

        {!skill.in_ssot && <MoveToSsotInline skillId={skill.id} />}

        <div className="flex items-center gap-2 text-[10.5px] font-mono text-text-muted mt-1.5">
          <span>{skill.file_count} 文件</span>
          <span className="opacity-40">·</span>
          <span>{formatBytes(skill.total_bytes)}</span>
          {skill.content_hash && (
            <>
              <span className="opacity-40">·</span>
              <span title="内容哈希 · 备份去重依据">#{skill.content_hash}</span>
            </>
          )}
          {skill.bindings.length > 0 && (
            <>
              <span className="opacity-40">·</span>
              <span>
                启用 {skill.bindings.filter(isBindingEnabled).length} / 共享于{" "}
                {skill.bindings.length}
              </span>
            </>
          )}
        </div>
      </div>

      <div className="flex-shrink-0 flex items-center gap-1.5">
        {report ? (
          <ScoreBadge report={report} />
        ) : (
          <button
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              onScan();
            }}
            disabled={scanning}
            className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-border-soft hover:border-text-muted disabled:opacity-50"
          >
            {scanning ? <Loader2 className="w-3 h-3 animate-spin" /> : <ShieldCheck className="w-3 h-3" />}
            扫描
          </button>
        )}
        <button
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onShowDetail();
          }}
          className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-border-soft hover:border-text-muted"
          title="查看文件树与 SKILL.md"
        >
          详情
          <ChevronRight className="w-3 h-3" />
        </button>
      </div>
    </label>
  );
}

export function ScoreBadge({ report }: { report: LocalSkillScanReport }) {
  const blocked = report.blocked;
  const color = blocked
    ? "rgb(var(--critical))"
    : report.score >= 80
      ? "rgb(var(--accent))"
      : report.score >= 50
        ? "rgb(var(--high))"
        : "rgb(var(--medium))";
  const Icon = blocked ? ShieldX : report.score >= 80 ? ShieldCheck : ShieldAlert;
  return (
    <div
      className="flex items-center gap-1.5 px-2 py-1 rounded font-mono text-[11px]"
      style={{ background: "color-mix(in srgb, " + color + " 12%, transparent)", color }}
    >
      <Icon className="w-3 h-3" />
      <span className="font-semibold">{report.score}</span>
    </div>
  );
}

function formatBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(2)} MB`;
  if (n >= 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${n} B`;
}
