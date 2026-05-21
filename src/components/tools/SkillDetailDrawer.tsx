import { useState } from "react";
import { X, FileText, Folder, Copy, Trash2, Loader2, AlertTriangle, ShieldCheck, ShieldX, ShieldAlert, MinusCircle, ChevronDown, ChevronRight } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import {
  useSkillDetail,
  useUninstallSkill,
  isBindingEnabled,
  bindingWarning,
  type SkillFile,
  type LocalSkillScanReport,
  type SkillFinding,
  type SkillRuleHit,
} from "@/hooks/useSkills";
import { UninstallSkillDialog } from "./UninstallSkillDialog";

interface Props {
  id: string | null;
  onClose: () => void;
  report?: LocalSkillScanReport;
  scanning?: boolean;
  onScan?: () => void;
}

type PreviewTabId = "skill_md" | "readme" | "scan";

export function SkillDetailDrawer({ id, onClose, report, scanning, onScan }: Props) {
  const { data, isLoading } = useSkillDetail(id);
  const uninstall = useUninstallSkill();
  const [preview, setPreview] = useState<PreviewTabId>("skill_md");
  const [uninstallOpen, setUninstallOpen] = useState(false);

  if (!id) return null;

  async function confirmUninstall() {
    if (!data) return;
    try {
      await uninstall.mutateAsync(data.meta.id);
      setUninstallOpen(false);
      onClose();
    } catch {
      /* toast in hook */
    }
  }

  return (
    <div className="fixed inset-0 z-40 flex" onClick={onClose}>
      {/* mask */}
      <div className="flex-1 bg-black/40 animate-fadein" />

      {/* drawer */}
      <aside
        className="w-[640px] max-w-[90vw] bg-bg border-l border-border shadow-2xl flex flex-col overflow-hidden animate-slidein-right"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <header className="px-5 py-3 border-b border-border-soft flex items-center gap-3 flex-shrink-0">
          <FileText className="w-4 h-4 text-accent flex-shrink-0" />
          <div className="flex-1 min-w-0">
            <div className="text-[14px] font-semibold tracking-tight truncate">
              {data?.meta.name ?? id}
            </div>
            <div className="text-[10.5px] font-mono text-text-muted truncate">
              .{data?.meta.source_agent ?? "—"} ·{" "}
              {data?.meta.version ? `v${data.meta.version}` : "无版本"} ·{" "}
              {data?.files.length ?? 0} 项
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-7 h-7 rounded-md hover:bg-bg-elev/60 flex items-center justify-center text-text-muted hover:text-text"
          >
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Path bar */}
        {data?.meta.source_path && (
          <div className="px-5 py-2 border-b border-border-soft flex items-center gap-2 bg-bg-elev/30">
            <span className="text-[10px] text-text-muted uppercase font-mono tracking-wider">路径</span>
            <code className="flex-1 truncate text-[11.5px] font-mono text-text-dim">
              {data.meta.source_path}
            </code>
            <button
              onClick={() => {
                navigator.clipboard.writeText(data.meta.source_path);
                toast.success("已复制路径");
              }}
              className="text-text-muted hover:text-text"
              title="复制"
            >
              <Copy className="w-3 h-3" />
            </button>
          </div>
        )}

        {/* 集中库状态 + Bindings + 卸载 */}
        {data && (
          <div className="px-5 py-2.5 border-b border-border-soft space-y-1.5">
            <div className="flex items-center gap-2 text-[11.5px]">
              <span className="text-text-muted uppercase tracking-wider text-[10px] font-mono">集中库</span>
              {data.meta.in_ssot ? (
                <span className="text-emerald-500 font-mono">● {data.meta.ssot_path}</span>
              ) : (
                <span className="text-amber-500 font-mono inline-flex items-center gap-1">
                  <AlertTriangle className="w-3 h-3" />
                  未纳入集中库
                </span>
              )}
              {data.meta.content_hash && (
                <span className="ml-auto font-mono text-text-muted text-[10.5px]">
                  #{data.meta.content_hash}
                </span>
              )}
            </div>

            {data.meta.bindings.length > 0 && (
              <div className="flex items-start gap-2 text-[11.5px]">
                <span className="text-text-muted uppercase tracking-wider text-[10px] font-mono pt-0.5">绑定</span>
                <div className="flex-1 flex flex-wrap gap-1">
                  {data.meta.bindings.map((b) => {
                    const enabled = isBindingEnabled(b);
                    const warn = bindingWarning(b);
                    const label = enabled
                      ? `✓ .${b.agent_name}`
                      : warn === "unmanaged"
                        ? `⚠ .${b.agent_name} (unmanaged)`
                        : warn === "external"
                          ? `⚠ .${b.agent_name} (external)`
                          : warn === "broken"
                            ? `✗ .${b.agent_name} (broken)`
                            : `· .${b.agent_name}`;
                    const color = enabled
                      ? "text-emerald-500"
                      : warn === "broken"
                        ? "text-critical"
                        : warn
                          ? "text-amber-500"
                          : "text-text-muted";
                    return (
                      <code key={b.agent_name} className={`font-mono ${color}`}>
                        {label}
                      </code>
                    );
                  })}
                </div>
              </div>
            )}

            {data.meta.in_ssot && (
              <div className="pt-1">
                <button
                  onClick={() => setUninstallOpen(true)}
                  disabled={uninstall.isPending}
                  className="flex items-center gap-1.5 text-[11.5px] px-2.5 py-1 rounded border border-critical/40 text-critical hover:bg-critical/5 disabled:opacity-50"
                >
                  {uninstall.isPending ? (
                    <Loader2 className="w-3 h-3 animate-spin" />
                  ) : (
                    <Trash2 className="w-3 h-3" />
                  )}
                  卸载（删集中库主副本 + 清所有 symlink）
                </button>
              </div>
            )}
          </div>
        )}

        {/* Body */}
        <div className="flex-1 overflow-hidden grid" style={{ gridTemplateColumns: "200px 1fr" }}>
          {/* 文件树 */}
          <div className="border-r border-border-soft overflow-auto py-2">
            {isLoading && (
              <div className="text-text-muted text-center py-4 text-[12px]">加载中…</div>
            )}
            {data?.files.map((f) => (
              <FileTreeRow key={f.path} file={f} />
            ))}
          </div>

          {/* 预览 */}
          <div className="flex flex-col overflow-hidden">
            {/* 预览切换 */}
            <div className="px-3 pt-2 flex items-center gap-1 text-[11.5px] border-b border-border-soft">
              <PreviewTab
                active={preview === "skill_md"}
                onClick={() => setPreview("skill_md")}
                badge={data?.skill_md ? "✓" : "—"}
              >
                SKILL.md
              </PreviewTab>
              <PreviewTab
                active={preview === "readme"}
                onClick={() => setPreview("readme")}
                badge={data?.readme ? "✓" : "—"}
              >
                README
              </PreviewTab>
              <PreviewTab
                active={preview === "scan"}
                onClick={() => setPreview("scan")}
                badge={
                  report
                    ? report.blocked
                      ? "✗"
                      : `${report.score}`
                    : "—"
                }
              >
                鉴定
              </PreviewTab>
            </div>

            <div className="flex-1 overflow-auto px-4 py-3">
              {preview === "skill_md" &&
                (data?.skill_md ? (
                  <pre className="font-mono text-[11.5px] leading-relaxed text-text-dim whitespace-pre-wrap break-words">
                    {data.skill_md}
                  </pre>
                ) : (
                  <EmptyPreview text="此技能未包含 SKILL.md" />
                ))}
              {preview === "readme" &&
                (data?.readme ? (
                  <pre className="font-mono text-[11.5px] leading-relaxed text-text-dim whitespace-pre-wrap break-words">
                    {data.readme}
                  </pre>
                ) : (
                  <EmptyPreview text="此技能未包含 README" />
                ))}
              {preview === "scan" && (
                <ScanPanel
                  report={report}
                  scanning={!!scanning}
                  onScan={onScan}
                />
              )}
            </div>
          </div>
        </div>
      </aside>

      {uninstallOpen && data && (
        <UninstallSkillDialog
          skill={data.meta}
          pending={uninstall.isPending}
          onConfirm={confirmUninstall}
          onCancel={() => setUninstallOpen(false)}
        />
      )}
    </div>
  );
}

function FileTreeRow({ file }: { file: SkillFile }) {
  return (
    <div
      className="flex items-center gap-1 px-2 py-0.5 hover:bg-bg-elev/40 text-[11.5px] cursor-default"
      style={{ paddingLeft: 8 + file.depth * 12 }}
      title={file.path}
    >
      {file.is_dir ? (
        <Folder className="w-3 h-3 text-text-muted flex-shrink-0" />
      ) : (
        <FileText className="w-3 h-3 text-text-muted flex-shrink-0" />
      )}
      <span className="flex-1 truncate font-mono">
        {file.path.split("/").pop()}
      </span>
      {!file.is_dir && (
        <span className="text-[10px] font-mono text-text-muted flex-shrink-0">
          {formatBytes(file.size)}
        </span>
      )}
    </div>
  );
}

function PreviewTab({
  children,
  active,
  onClick,
  badge,
}: {
  children: React.ReactNode;
  active: boolean;
  onClick: () => void;
  badge?: string;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "px-2.5 py-1.5 rounded-md font-medium transition-colors flex items-center gap-1.5",
        active ? "bg-bg-elev/70 text-text" : "text-text-muted hover:text-text",
      )}
    >
      <span>{children}</span>
      {badge && <span className="text-[9.5px] font-mono opacity-70">{badge}</span>}
    </button>
  );
}

function EmptyPreview({ text }: { text: string }) {
  return (
    <div className="text-text-muted text-[12px] text-center py-12">{text}</div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 鉴定 panel：评分依据 + findings + 硬触发
// ──────────────────────────────────────────────────────────────────
function ScanPanel({
  report,
  scanning,
  onScan,
}: {
  report?: LocalSkillScanReport;
  scanning: boolean;
  onScan?: () => void;
}) {
  if (!report) {
    return (
      <div className="text-center py-12">
        <div
          className="inline-flex w-10 h-10 rounded-xl items-center justify-center mb-3"
          style={{
            background: "color-mix(in srgb, rgb(var(--accent)) 12%, transparent)",
            color: "rgb(var(--accent))",
          }}
        >
          <ShieldCheck className="w-5 h-5" />
        </div>
        <div className="text-[12.5px] text-text-dim mb-3 max-w-xs mx-auto leading-relaxed">
          这个技能还没扫描过。点下方按钮跑 SkillGuard 规则集，结果会给出评分与命中规则。
        </div>
        {onScan ? (
          <button
            onClick={onScan}
            disabled={scanning}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12px] font-medium border border-accent/40 text-accent hover:bg-accent/5 disabled:opacity-50"
          >
            {scanning ? <Loader2 className="w-3 h-3 animate-spin" /> : <ShieldCheck className="w-3 h-3" />}
            立即扫描
          </button>
        ) : (
          <div className="text-[11px] text-text-muted">在列表行点「扫描」即可</div>
        )}
      </div>
    );
  }

  // ── 顶部评分卡 ─────────────────────────────────────────────
  const blocked = report.blocked;
  const color = blocked
    ? "rgb(var(--critical))"
    : report.score >= 80
      ? "rgb(var(--accent))"
      : report.score >= 50
        ? "rgb(var(--high))"
        : "rgb(var(--medium))";
  const HeaderIcon = blocked ? ShieldX : report.score >= 80 ? ShieldCheck : ShieldAlert;
  const verdictLabel = blocked
    ? "已阻止 · 含硬触发规则"
    : report.score >= 80
      ? "安全 · 加权扣分较少"
      : report.score >= 50
        ? "警告 · 多条规则命中"
        : "高风险 · 接近阻止阈值";

  // ── 累计扣分 / 命中规则 ───────────────────────────────────
  const totalDeduction = report.findings.reduce(
    (s, f) => s + f.weighted_deduction,
    0,
  );

  return (
    <div className="space-y-4">
      {/* 评分头部 */}
      <div
        className="rounded-xl border p-4 flex items-center gap-4"
        style={{
          borderColor: `color-mix(in srgb, ${color} 35%, transparent)`,
          background: `color-mix(in srgb, ${color} 6%, transparent)`,
        }}
      >
        <div
          className="w-14 h-14 rounded-xl flex items-center justify-center flex-shrink-0"
          style={{
            background: `color-mix(in srgb, ${color} 18%, transparent)`,
            color,
          }}
        >
          <HeaderIcon className="w-7 h-7" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-baseline gap-2">
            <span
              className="font-mono text-[28px] font-bold leading-none tabular-nums"
              style={{ color }}
            >
              {blocked ? 0 : report.score}
            </span>
            <span className="text-text-muted text-[11.5px] font-mono">/ 100</span>
          </div>
          <div className="text-[12px] mt-1" style={{ color }}>
            {verdictLabel}
          </div>
        </div>
      </div>

      {/* 评分依据 */}
      <Section title="评分公式">
        <div className="space-y-1 text-[11.5px] text-text-dim leading-relaxed">
          <Step label="起始" value={blocked ? "—" : "100"} />
          {blocked ? (
            <Step
              label="硬触发归零"
              value={`命中 ${report.hard_triggers.length} 项 → score = 0, blocked = true`}
              color="rgb(var(--critical))"
            />
          ) : (
            <Step
              label="加权扣分"
              value={`${report.findings.length} 条规则命中 · 共扣 ${totalDeduction}`}
            />
          )}
          {!blocked && (
            <Step
              label="终值"
              value={`${report.score}`}
              color="rgb(var(--accent))"
            />
          )}
        </div>
      </Section>

      {/* 硬触发规则（如有） */}
      {report.hard_triggers.length > 0 && (
        <Section title={`硬触发规则 (${report.hard_triggers.length})`}>
          <div className="space-y-1.5">
            {report.hard_triggers.map((hit) => (
              <RuleHitCard key={hit.rule_id} hit={hit} kind="hard" />
            ))}
          </div>
        </Section>
      )}

      {/* 加权 findings */}
      {report.findings.length > 0 ? (
        <Section title={`加权扣分明细 (${report.findings.length})`}>
          <div className="space-y-1.5">
            {report.findings.map((f) => (
              <FindingCard key={f.rule_id} finding={f} />
            ))}
          </div>
        </Section>
      ) : !blocked && report.hard_triggers.length === 0 ? (
        <Section title="加权扣分明细">
          <div className="text-[12px] text-emerald-500 flex items-center gap-1.5">
            <ShieldCheck className="w-3.5 h-3.5" />
            未命中任何加权规则 · 100 分全保留
          </div>
        </Section>
      ) : null}

      {/* 算法说明（折叠） */}
      <details className="rounded-md border border-border-soft">
        <summary className="cursor-pointer px-3 py-1.5 text-[11.5px] text-text-muted hover:text-text font-medium">
          算法说明
        </summary>
        <div className="px-3 pb-2.5 text-[11px] text-text-dim leading-relaxed space-y-1">
          <p>
            起始分 100。命中任意「硬触发」规则（如 eval(input)、同形字、分阶段载荷）
            立即归零并标记为 blocked。
          </p>
          <p>
            「加权扣分」按公式 <code className="font-mono text-text-muted">d = 2w × (1 − 0.5ⁿ)</code>
            （w = 规则权重，n = 命中次数）累计扣减，最低 0 分。
          </p>
          <p>
            score &lt; 30 时也会被自动 block，由前端「危险」过滤分类。
          </p>
        </div>
      </details>
    </div>
  );
}

/** 硬触发规则卡片（始终展开，红色） */
function RuleHitCard({
  hit,
  kind,
}: {
  hit: SkillRuleHit;
  kind: "hard";
}) {
  const accent = kind === "hard" ? "rgb(var(--critical))" : "rgb(245 158 11)";
  return (
    <div
      className="rounded-md border px-3 py-2.5"
      style={{
        background: "color-mix(in srgb, " + accent + " 6%, transparent)",
        borderColor: "color-mix(in srgb, " + accent + " 35%, transparent)",
      }}
    >
      <div className="flex items-center gap-2 mb-1.5">
        <ShieldX className="w-3.5 h-3.5 flex-shrink-0" style={{ color: accent }} />
        <code className="font-mono font-semibold text-[12px]" style={{ color: accent }}>
          {hit.rule_id}
        </code>
        <span className="text-text text-[12.5px] font-medium">{hit.description}</span>
        <span className="ml-auto text-[10.5px] font-mono text-text-muted">
          命中即归零
        </span>
      </div>
      <RuleBody hit={hit} />
    </div>
  );
}

/** 加权 finding 卡片（默认折叠摘要 + 点击展开） */
function FindingCard({ finding }: { finding: SkillFinding }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="rounded-md border border-border-soft overflow-hidden">
      <button
        onClick={() => setOpen((v) => !v)}
        className="w-full grid gap-2 px-3 py-1.5 items-center text-[11.5px] hover:bg-bg-elev/30 transition-colors"
        style={{ gridTemplateColumns: "20px 80px 1fr 60px 50px" }}
      >
        {open ? (
          <ChevronDown className="w-3 h-3 text-text-muted" />
        ) : (
          <ChevronRight className="w-3 h-3 text-text-muted" />
        )}
        <code className="font-mono text-text text-left">{finding.rule_id}</code>
        <span className="text-text-dim leading-snug text-left truncate">
          {finding.description}
        </span>
        <span className="text-right font-mono text-text-muted">
          ×{finding.match_count}
        </span>
        <span
          className="text-right font-mono font-semibold"
          style={{ color: "rgb(245 158 11)" }}
        >
          -{finding.weighted_deduction}
        </span>
      </button>
      {open && (
        <div className="px-3 pb-3 pt-1 bg-bg-elev/20 border-t border-border-soft">
          <RuleBody hit={finding} />
        </div>
      )}
    </div>
  );
}

/** 规则元数据展示（为何 / 命中片段 / 示例 / 修复） */
function RuleBody({ hit }: { hit: SkillRuleHit }) {
  return (
    <div className="space-y-2 text-[11.5px] leading-relaxed">
      <div>
        <div className="text-[10px] text-text-muted uppercase tracking-wider font-mono mb-0.5">
          为何危险
        </div>
        <div className="text-text-dim">{hit.why}</div>
      </div>
      {hit.matched_needles.length > 0 && (
        <div>
          <div className="text-[10px] text-text-muted uppercase tracking-wider font-mono mb-0.5">
            命中关键字
          </div>
          <div className="flex flex-wrap gap-1">
            {hit.matched_needles.map((n) => (
              <code
                key={n}
                className="font-mono text-[10.5px] px-1.5 py-0.5 rounded bg-bg-elev/60 border border-border-soft text-text"
              >
                {n}
              </code>
            ))}
          </div>
        </div>
      )}
      {hit.example && (
        <div>
          <div className="text-[10px] text-text-muted uppercase tracking-wider font-mono mb-0.5">
            示例片段
          </div>
          <pre className="font-mono text-[11px] leading-snug bg-bg/70 border border-border-soft rounded-md px-2.5 py-1.5 text-text-dim whitespace-pre-wrap break-words">
            {hit.example}
          </pre>
        </div>
      )}
      {hit.remediation && (
        <div>
          <div className="text-[10px] text-text-muted uppercase tracking-wider font-mono mb-0.5">
            修复建议
          </div>
          <div className="text-text">{hit.remediation}</div>
        </div>
      )}
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="text-[10px] text-text-muted uppercase tracking-wider font-bold font-mono mb-1.5">
        {title}
      </div>
      {children}
    </div>
  );
}

function Step({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color?: string;
}) {
  return (
    <div className="flex items-center justify-between px-2.5 py-1 rounded bg-bg-elev/30">
      <span className="text-text-muted">{label}</span>
      <span className="font-mono font-medium" style={color ? { color } : undefined}>
        {value}
      </span>
    </div>
  );
}

function formatBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)}M`;
  if (n >= 1024) return `${(n / 1024).toFixed(1)}K`;
  return `${n}B`;
}
