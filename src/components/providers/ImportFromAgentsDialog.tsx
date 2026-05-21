import { useEffect, useState } from "react";
import {
  X,
  Loader2,
  Sparkles,
  AlertTriangle,
  ScanSearch,
  Info,
  KeyRound,
  Lock,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import {
  useImportCandidates,
  useUnmanagedAgents,
  useBulkImportProfiles,
  type ImportCandidate,
  type ProviderKind,
  type UnmanagedAgent,
} from "@/hooks/useProviders";

interface Props {
  onClose: () => void;
  onImported?: (count: number) => void;
  /** 用户点击「未托管」区的「手动建 Profile」时回调，关闭本 Dialog 并打开新建表单 */
  onCreateManualProfile?: (hint?: { platform?: string; agent_name?: string }) => void;
}

export function ImportFromAgentsDialog({ onClose, onImported, onCreateManualProfile }: Props) {
  const { data: candidates = [], isLoading, refetch } = useImportCandidates(true);
  const { data: unmanaged = [], refetch: refetchUnmanaged } = useUnmanagedAgents(true);
  const bulkImport = useBulkImportProfiles();
  const [selected, setSelected] = useState<Set<string>>(new Set());

  // 默认勾选所有未冲突的候选
  useEffect(() => {
    if (candidates.length > 0 && selected.size === 0) {
      setSelected(
        new Set(
          candidates
            .filter((c) => !c.conflicts_with_existing_profile)
            .map((c) => c.candidate_id),
        ),
      );
    }
  }, [candidates, selected.size]);

  const importable = candidates.filter((c) => !c.conflicts_with_existing_profile);
  const allChecked =
    importable.length > 0 &&
    importable.every((c) => selected.has(c.candidate_id));

  function toggle(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelected(next);
  }

  function toggleAll() {
    if (allChecked) {
      setSelected(new Set());
    } else {
      setSelected(new Set(importable.map((c) => c.candidate_id)));
    }
  }

  async function handleImport() {
    if (selected.size === 0) return;
    try {
      const result = await bulkImport.mutateAsync({
        candidateIds: Array.from(selected),
        setFirstAsDefault: true,
      });
      const n = result.created.length;
      if (n > 0) {
        toast.success(`已导入 ${n} 个模型渠道`);
        onImported?.(n);
        onClose();
      }
      if (result.skipped.length > 0) {
        toast.warning(`${result.skipped.length} 项跳过`);
      }
    } catch {
      // toast 已在 hook
    }
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-2xl max-h-[85vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden">
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            <ScanSearch className="w-4 h-4 text-accent" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              从已发现 Agent 导入模型配置
            </h3>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Privacy banner */}
        <div className="px-5 py-2.5 bg-bg-elev/40 border-b border-border-soft">
          <div className="flex items-start gap-2 text-[11.5px] text-text-dim leading-relaxed">
            <Info className="w-3.5 h-3.5 mt-0.5 flex-shrink-0 text-accent" />
            <span>
              ClawHeart 已从你电脑上的 Agent 配置文件读取了中转 API 凭据。
              真实 key 仅在本机内存中临时存在，导入后立即加密存入 macOS Keychain；
              界面与日志只显示掩码。点击「导入」后才会持久化。
            </span>
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-auto px-5 py-4 space-y-5">
          {isLoading && (
            <div className="flex items-center gap-2 text-[12.5px] text-text-muted py-4">
              <Loader2 className="w-4 h-4 animate-spin" />
              正在扫描已发现 Agent 的配置文件…
            </div>
          )}

          {!isLoading && candidates.length === 0 && unmanaged.length === 0 && (
            <EmptyState onRescan={() => { refetch(); refetchUnmanaged(); }} />
          )}

          {/* ========= 可导入凭据 ========= */}
          {!isLoading && candidates.length > 0 && (
            <section>
              <SectionHeader
                icon={<KeyRound className="w-3.5 h-3.5 text-emerald-500" />}
                title="可导入凭据"
                subtitle="从 Agent 配置文件中读取到完整 API key"
                count={candidates.length}
                onRescan={() => { refetch(); refetchUnmanaged(); }}
              />

              <div className="flex items-center mb-2 px-1">
                <label className="flex items-center gap-2 text-[12px] cursor-pointer">
                  <input
                    type="checkbox"
                    checked={allChecked}
                    onChange={toggleAll}
                    className="w-3.5 h-3.5 accent-accent"
                  />
                  <span className="text-text-dim">
                    全选 ({selected.size}/{candidates.length})
                  </span>
                </label>
              </div>

              <div className="space-y-2">
                {candidates.map((c) => (
                  <CandidateCard
                    key={c.candidate_id}
                    candidate={c}
                    selected={selected.has(c.candidate_id)}
                    onToggle={() => toggle(c.candidate_id)}
                  />
                ))}
              </div>
            </section>
          )}

          {/* ========= 检测到但无法自动导入 ========= */}
          {!isLoading && unmanaged.length > 0 && (
            <section>
              <SectionHeader
                icon={<Lock className="w-3.5 h-3.5 text-amber-500" />}
                title="检测到但无法自动导入"
                subtitle="凭据走 OAuth / 环境变量 / 已托管 — 仍可手动建渠道后用「一键覆盖」托管"
                count={unmanaged.length}
              />

              <div className="space-y-2">
                {unmanaged.map((u) => (
                  <UnmanagedCard
                    key={u.agent_id}
                    agent={u}
                    onCreateProfile={
                      onCreateManualProfile
                        ? () =>
                            onCreateManualProfile({
                              platform: u.agent_platform,
                              agent_name: u.agent_name,
                            })
                        : undefined
                    }
                  />
                ))}
              </div>
            </section>
          )}
        </div>

        {/* Footer */}
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center justify-between">
          <div className="text-[11px] text-text-muted">
            {selected.size > 0 && `已选 ${selected.size} 个候选`}
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3.5 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
            >
              取消
            </button>
            <button
              onClick={handleImport}
              disabled={selected.size === 0 || bulkImport.isPending}
              className="flex items-center gap-1.5 px-4 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
            >
              {bulkImport.isPending ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : (
                <Sparkles className="w-3.5 h-3.5" />
              )}
              导入 {selected.size} 个模型渠道
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 候选卡片
// ──────────────────────────────────────────────────────────────────
function CandidateCard({
  candidate,
  selected,
  onToggle,
}: {
  candidate: ImportCandidate;
  selected: boolean;
  onToggle: () => void;
}) {
  const conflict = candidate.conflicts_with_existing_profile;
  const disabled = conflict;

  return (
    <label
      className={cn(
        "flex items-start gap-2.5 p-3 rounded-md border transition-colors",
        disabled
          ? "border-border-soft bg-bg-elev/30 opacity-60 cursor-not-allowed"
          : selected
            ? "border-accent bg-accent/5 cursor-pointer"
            : "border-border bg-bg-elev hover:border-text-muted cursor-pointer",
      )}
    >
      <input
        type="checkbox"
        checked={selected}
        onChange={onToggle}
        disabled={disabled}
        className="mt-1 w-3.5 h-3.5 accent-accent flex-shrink-0"
      />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-1">
          <span className="text-[13px] font-medium text-text">
            {candidate.suggested_name}
          </span>
          <KindBadge kind={candidate.inferred_kind} />
          {conflict && (
            <span className="inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300">
              <AlertTriangle className="w-2.5 h-2.5" />
              已存在渠道：{candidate.existing_profile_name}
            </span>
          )}
        </div>

        <div className="grid grid-cols-[60px_1fr] gap-x-2 gap-y-0.5 text-[11px] font-mono">
          <span className="text-text-muted">Base URL</span>
          <span className="text-text-dim truncate">{candidate.base_url}</span>
          <span className="text-text-muted">API Key</span>
          <span className="text-text-dim truncate">{candidate.api_key_masked}</span>
          <span className="text-text-muted">协议</span>
          <span className="text-text-dim">{candidate.inferred_protocol}</span>
        </div>

        <div className="mt-1.5 flex flex-wrap items-center gap-1">
          {candidate.source_labels.map((label, i) => (
            <span
              key={i}
              className="inline-flex items-center gap-1 text-[10.5px] px-1.5 py-0.5 rounded bg-bg-elev2 text-text-muted border border-border-soft"
            >
              📄 {label}
            </span>
          ))}
          {candidate.source_agents.length > 1 && (
            <span className="text-[10.5px] text-text-muted">
              · 与 {candidate.source_agents.length - 1} 个其他 Agent 共享
            </span>
          )}
        </div>
      </div>
    </label>
  );
}

function KindBadge({ kind }: { kind: ProviderKind }) {
  const map: Record<ProviderKind, { label: string; color: string }> = {
    openrouter: { label: "OpenRouter", color: "indigo" },
    openai: { label: "OpenAI", color: "emerald" },
    anthropic: { label: "Anthropic", color: "orange" },
    azure: { label: "Azure", color: "blue" },
    deepbricks: { label: "DeepBricks", color: "purple" },
    newapi: { label: "NewAPI", color: "cyan" },
    litellm: { label: "LiteLLM 本地", color: "slate" },
    custom: { label: "自定义", color: "zinc" },
  };
  const meta = map[kind];
  const cls = `bg-${meta.color}-500/15 text-${meta.color}-700 dark:text-${meta.color}-300`;
  return (
    <span className={cn("text-[10px] px-1.5 py-0.5 rounded font-medium", cls)}>
      {meta.label}
    </span>
  );
}

function EmptyState({ onRescan }: { onRescan: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 text-center">
      <ScanSearch className="w-10 h-10 text-text-muted mb-3" />
      <div className="text-[13px] font-medium mb-1">未发现任何 Agent</div>
      <div className="text-[11.5px] text-text-dim max-w-sm leading-relaxed mb-4">
        ClawHeart 在本机未发现 Claude Code / Codex / Cursor / Continue / OpenClaw 等 Agent。
        启动这些工具后回来「重新扫描」。
      </div>
      <button
        onClick={onRescan}
        className="px-3.5 py-1.5 rounded-md text-[12px] border border-border hover:border-text-muted"
      >
        重新扫描
      </button>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 分区头
// ──────────────────────────────────────────────────────────────────
function SectionHeader({
  icon,
  title,
  subtitle,
  count,
  onRescan,
}: {
  icon: React.ReactNode;
  title: string;
  subtitle?: string;
  count: number;
  onRescan?: () => void;
}) {
  return (
    <div className="mb-2 flex items-start justify-between gap-3">
      <div className="flex items-start gap-2 flex-1 min-w-0">
        <span className="mt-0.5">{icon}</span>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5">
            <span className="text-[12.5px] font-semibold tracking-tight">{title}</span>
            <span className="text-[10.5px] font-mono text-text-muted px-1.5 py-0.5 rounded bg-bg-elev/50">
              {count}
            </span>
          </div>
          {subtitle && (
            <p className="text-[11px] text-text-dim leading-snug mt-0.5">{subtitle}</p>
          )}
        </div>
      </div>
      {onRescan && (
        <button
          onClick={onRescan}
          className="text-[11px] text-text-muted hover:text-text flex-shrink-0"
        >
          重新扫描
        </button>
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 未托管 Agent 卡片
// ──────────────────────────────────────────────────────────────────
function UnmanagedCard({
  agent,
  onCreateProfile,
}: {
  agent: UnmanagedAgent;
  onCreateProfile?: () => void;
}) {
  const isProxied = agent.reason === "already_proxied";
  const color = isProxied ? "emerald" : "amber";
  return (
    <div
      className={cn(
        "p-3 rounded-md border bg-bg-elev/30",
        isProxied
          ? "border-emerald-500/30"
          : "border-amber-500/30",
      )}
    >
      <div className="flex items-start gap-2.5">
        <div
          className={cn(
            "w-7 h-7 rounded-md flex items-center justify-center flex-shrink-0 mt-0.5",
            isProxied ? "bg-emerald-500/15 text-emerald-500" : "bg-amber-500/15 text-amber-500",
          )}
        >
          {isProxied ? <Sparkles className="w-3.5 h-3.5" /> : <Lock className="w-3.5 h-3.5" />}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap mb-1">
            <span className="text-[13px] font-medium text-text">{agent.agent_name}</span>
            <span className="text-[10px] font-mono text-text-muted px-1.5 py-0.5 rounded bg-bg-elev/50">
              .{agent.agent_platform}
            </span>
            <span
              className={cn(
                "text-[10px] px-1.5 py-0.5 rounded font-medium",
                `bg-${color}-500/15 text-${color}-700 dark:text-${color}-300`,
              )}
            >
              {agent.reason_label}
            </span>
          </div>
          <p className="text-[11.5px] text-text-dim leading-relaxed">{agent.hint}</p>
        </div>
        {!isProxied && onCreateProfile && (
          <button
            onClick={onCreateProfile}
            className="flex-shrink-0 px-2.5 py-1 rounded-md text-[11.5px] font-medium border border-border bg-bg-elev hover:border-accent hover:text-accent transition-colors"
          >
            手动建渠道 →
          </button>
        )}
      </div>
    </div>
  );
}
