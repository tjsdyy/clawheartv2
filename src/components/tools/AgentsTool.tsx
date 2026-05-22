/**
 * 本机 Agent —— 一站式管理入口。
 *
 * 结构（参考 cc-switch 风格）：
 * ┌──────────────────────────────────────────────────────────┐
 * │ [Claude][Codex][Gemini]...    ↻  +                       │ ← Agent tab + 操作
 * ├──────────────────────────────────────────────────────────┤
 * │ ● GLM                                                    │
 * │   https://open.bigmodel.cn                  [启用]       │
 * │                                                          │
 * │ ● Anthropic 直连  [当前使用]                             │
 * │   https://api.anthropic.com                  [已应用]    │
 * │                                                          │
 * │ ...                                                      │
 * │                                                          │
 * │ ── 候选 Agent（2）                              [展开]   │
 * └──────────────────────────────────────────────────────────┘
 *
 * 设计要点：
 * - 顶部 tab 切 Agent（已纳入管理的），不显示候选
 * - 列表项是「全局渠道列表」，每行显示该渠道是否为当前 tab Agent 的生效渠道
 * - 一键启用 → 弹 OverwriteWizard 锁定当前 Agent + 渠道（已有 initialAgentId 模式）
 * - 候选 Agent 折叠在底部，确认后自动出现在 tab 上
 */

import { useEffect, useMemo, useState } from "react";
import {
  RefreshCw,
  Plus,
  CheckCircle2,
  AlertTriangle,
  ScanSearch,
  ChevronDown,
  ChevronRight,
  ShieldCheck,
  X,
  Loader2,
  Settings,
  History,
  Zap,
  Download,
} from "lucide-react";
import { useQueries } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import {
  useAgents,
  useRediscoverAgents,
  useConfirmUnknownAgent,
  useIgnoreUnknownAgent,
  type DiscoveredAgent,
} from "@/hooks/useAgents";
import {
  useApplyBatches,
  useApplyRealStatus,
  useRollbackSnapshot,
  listBatchSnapshots,
  type SnapshotDto,
} from "@/hooks/useAgentConfig";
import {
  useProviderProfiles,
  type ProviderProfile,
} from "@/hooks/useProviders";
import { OverwriteWizard } from "@/components/providers/OverwriteWizard";
import { AddChannelDialog } from "@/components/agents/AddChannelDialog";
import { BrandIcon } from "@/components/agents/BrandIcon";
import { AgentHistoryDrawer } from "@/components/agents/AgentHistoryDrawer";
import { SelectChannelDialog } from "@/components/agents/SelectChannelDialog";
import { ImportFromAgentDialog } from "@/components/agents/ImportFromAgentDialog";
import { useAgentChannels } from "@/hooks/useChannelAssignments";
import { toast } from "sonner";
import { PROVIDER_PRESETS, type ProviderPreset } from "@/data/provider-presets";

const PLATFORM_LABELS: Record<string, string> = {
  claude: "Claude",
  codex: "Codex",
  cursor: "Cursor",
  gemini: "Gemini",
  windsurf: "Windsurf",
  openclaw: "OpenClaw",
  openeva: "OpenEva",
};

function platformLabel(p: string): string {
  if (p.startsWith("unknown:")) return p.slice(8);
  return PLATFORM_LABELS[p] ?? p;
}

/** 判断协议是否兼容某 Agent 平台 */
function matchesPlatform(protocol: string, platform: string): boolean {
  if (platform === "claude") return protocol === "anthropic";
  if (platform === "codex") return protocol === "openai" || protocol === "openai_responses";
  if (platform === "gemini") return protocol === "gemini";
  if (platform === "cursor" || platform === "windsurf") {
    return protocol === "openai" || protocol === "anthropic";
  }
  // openclaw / openeva / unknown:* —— 通用兼容
  return true;
}

const PROTOCOL_LABELS: Record<string, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  openai_responses: "OpenAI Resp.",
  gemini: "Gemini",
  ollama: "Ollama",
};

function platformColorKey(platform: string): string {
  switch (platform) {
    case "claude":
      return "monitor";
    case "codex":
      return "scan";
    case "cursor":
      return "skills";
    case "gemini":
      return "openclaw";
    case "windsurf":
      return "logs";
    case "openclaw":
      return "openclaw";
    case "openeva":
      return "audit";
    default:
      return "monitor";
  }
}

// ──────────────────────────────────────────────────────────────────
// 主组件
// ──────────────────────────────────────────────────────────────────
export function AgentsTool() {
  const { t } = useTranslation();
  const { data: agents = [], isLoading } = useAgents();
  const { data: profiles = [] } = useProviderProfiles();
  const rediscover = useRediscoverAgents();

  const managed = useMemo(
    () => agents.filter((a) => a.status !== "candidate"),
    [agents],
  );
  const candidates = useMemo(
    () => agents.filter((a) => a.status === "candidate"),
    [agents],
  );

  const [activeAgentId, setActiveAgentId] = useState<string | null>(null);
  const [wizardProfileId, setWizardProfileId] = useState<string | null>(null);
  const [candidatesOpen, setCandidatesOpen] = useState(false);
  const [addOpen, setAddOpen] = useState(false);
  const [selectOpen, setSelectOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [editingProfileId, setEditingProfileId] = useState<string | null>(null);

  // 当前活跃 Agent 已分配的渠道 ID 列表
  const { data: assignedIds = [] } = useAgentChannels(activeAgentId);
  const assignedSet = new Set(assignedIds);

  const editingProfile = editingProfileId
    ? profiles.find((p) => p.id === editingProfileId) ?? null
    : null;

  // 自动选第一个 Agent
  useEffect(() => {
    if (!activeAgentId && managed.length > 0) {
      const first = managed[0];
      setActiveAgentId(`${first.platform}/${first.agent_name}`);
    }
    // 切换 Agent 后若不再存在，回退到第一个
    if (
      activeAgentId &&
      !managed.find(
        (a) => `${a.platform}/${a.agent_name}` === activeAgentId,
      )
    ) {
      setActiveAgentId(
        managed.length > 0
          ? `${managed[0].platform}/${managed[0].agent_name}`
          : null,
      );
    }
  }, [managed, activeAgentId]);

  const activeAgent = managed.find(
    (a) => `${a.platform}/${a.agent_name}` === activeAgentId,
  );
  const channelsMap = useAgentCurrentChannels();
  const currentChannelName = activeAgentId
    ? channelsMap.get(activeAgentId) ?? null
    : null;

  // 协议匹配：判断渠道是否兼容当前 Agent
  function isProtocolCompatible(p: ProviderProfile): boolean {
    if (!activeAgent) return true;
    return matchesPlatform(p.protocol, activeAgent.platform);
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header: Agent tabs + actions */}
      <header className="px-5 py-2.5 border-b border-border-soft flex items-center gap-3 flex-shrink-0">
        <div className="flex-1 flex items-center gap-1 overflow-x-auto scrollbar-hidden">
          {managed.length === 0 ? (
            <span className="text-[12px] text-text-muted py-1">
              暂无 Agent · 点击刷新扫描本机
            </span>
          ) : (
            managed.map((a) => {
              const id = `${a.platform}/${a.agent_name}`;
              const isActive = id === activeAgentId;
              return (
                <AgentTab
                  key={id}
                  agent={a}
                  active={isActive}
                  taken={channelsMap.has(id)}
                  onClick={() => setActiveAgentId(id)}
                />
              );
            })
          )}
        </div>
        <button
          onClick={() => rediscover.mutate()}
          disabled={rediscover.isPending}
          className="flex items-center gap-1.5 px-2 py-1 rounded-md text-[11.5px] text-text-muted hover:text-text border border-border hover:border-text-muted disabled:opacity-50"
          title="重新扫描本机 Agent"
        >
          <RefreshCw
            className={cn(
              "w-3.5 h-3.5",
              rediscover.isPending && "animate-spin",
            )}
          />
          {t("agents.rediscover")}
        </button>
      </header>

      {/* Channel list */}
      <div className="flex-1 overflow-auto px-5 py-4 space-y-2">
        {isLoading ? (
          <div className="text-text-muted text-[13px] text-center py-12">
            加载中…
          </div>
        ) : !activeAgent ? (
          <div className="text-text-muted text-[13px] text-center py-12">
            未发现已纳入管理的 Agent。
          </div>
        ) : (
          <AssignedChannelList
            profiles={profiles.filter((p) => assignedSet.has(p.id))}
            agent={activeAgent}
            currentChannelName={currentChannelName}
            isProtocolCompatible={isProtocolCompatible}
            onSelectFromLibrary={() => setSelectOpen(true)}
            onCreateNew={() => setAddOpen(true)}
            onOpenHistory={() => setHistoryOpen(true)}
            onOpenImport={() => setImportOpen(true)}
            onApply={(id) => setWizardProfileId(id)}
            onEdit={(id) => setEditingProfileId(id)}
          />
        )}

        {/* 候选 Agent 折叠区 */}
        {candidates.length > 0 && (
          <section className="mt-6 pt-4 border-t border-border-soft">
            <button
              onClick={() => setCandidatesOpen((v) => !v)}
              className="w-full flex items-center gap-2 text-[12px] text-text-muted hover:text-text"
            >
              {candidatesOpen ? (
                <ChevronDown className="w-3.5 h-3.5" />
              ) : (
                <ChevronRight className="w-3.5 h-3.5" />
              )}
              <ScanSearch className="w-3.5 h-3.5" />
              <span>候选 Agent · {candidates.length}</span>
              <span className="text-[10.5px] text-text-muted">
                （在 ~/.xxx/ 发现 AI 线索，待确认）
              </span>
            </button>
            {candidatesOpen && (
              <div className="mt-3 space-y-2">
                {candidates.map((c) => (
                  <CandidateRow key={`${c.platform}-${c.agent_name}`} agent={c} />
                ))}
              </div>
            )}
          </section>
        )}
      </div>

      {/* Apply wizard（锁定当前 Agent） */}
      {wizardProfileId && activeAgent && (
        <OverwriteWizard
          initialProfileId={wizardProfileId}
          initialAgentId={`${activeAgent.platform}/${activeAgent.agent_name}`}
          onClose={() => setWizardProfileId(null)}
          onApplied={() => setWizardProfileId(null)}
        />
      )}

      {/* 新增渠道（cc-switch 风格的 Provider Preset 面板） */}
      {addOpen && (
        <AddChannelDialog
          onClose={() => setAddOpen(false)}
          recommendedPlatform={activeAgent?.platform}
          autoAssignToAgent={activeAgentId ?? undefined}
        />
      )}

      {/* 编辑渠道 */}
      {editingProfile && (
        <AddChannelDialog
          onClose={() => setEditingProfileId(null)}
          editingProfile={editingProfile}
        />
      )}

      {/* 选择渠道（从库分配给当前 Agent） */}
      {selectOpen && activeAgent && (
        <SelectChannelDialog
          agentId={`${activeAgent.platform}/${activeAgent.agent_name}`}
          agentName={activeAgent.agent_name}
          agentPlatform={activeAgent.platform}
          isCompatible={isProtocolCompatible}
          onClose={() => setSelectOpen(false)}
          onCreateNew={() => {
            setSelectOpen(false);
            setAddOpen(true);
          }}
        />
      )}

      {/* 托管历史抽屉 */}
      {historyOpen && activeAgent && (
        <AgentHistoryDrawer
          agent={activeAgent}
          onClose={() => setHistoryOpen(false)}
        />
      )}

      {/* 从 Agent 配置反向导入渠道 */}
      {importOpen && activeAgent && (
        <ImportFromAgentDialog
          agent={activeAgent}
          onClose={() => setImportOpen(false)}
        />
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Agent tab 按钮
// ──────────────────────────────────────────────────────────────────
function AgentTab({
  agent,
  active,
  taken,
  onClick,
}: {
  agent: DiscoveredAgent;
  active: boolean;
  /** 该 agent 是否已被 ClawHeart 托管 */
  taken: boolean;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  const colorKey = platformColorKey(agent.platform);
  const initial = agent.agent_name.charAt(0);
  const importable = IMPORTABLE_PLATFORMS.has(agent.platform);
  return (
    <button
      onClick={onClick}
      className={cn(
        "relative flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12.5px] font-medium whitespace-nowrap transition-colors",
        active
          ? "bg-bg-elev2 text-text shadow-sm"
          : "text-text-muted hover:text-text hover:bg-bg-elev",
      )}
    >
      <span
        className={cn(
          "w-5 h-5 rounded flex items-center justify-center text-[10px] font-bold font-mono",
        )}
        style={{
          background: active
            ? `rgb(var(--tool-${colorKey}) / 0.18)`
            : `rgb(var(--tool-${colorKey}) / 0.10)`,
          color: `rgb(var(--tool-${colorKey}))`,
        }}
      >
        {initial}
      </span>
      <span>{platformLabel(agent.platform)}</span>
      {/* 可导入角标：内联，明显 accent 色 chip + 白色文字 */}
      {importable && (
        <span
          className="inline-flex items-center px-1.5 h-[15px] rounded-full text-[9px] font-bold text-white shadow-sm leading-none whitespace-nowrap flex-shrink-0"
          style={{ background: "rgb(var(--accent))" }}
          title={t("agents.badge_importable")}
        >
          {t("agents.badge_importable")}
        </span>
      )}
      {/* 托管状态点：绿色=已托管，灰色=未托管 */}
      <span
        className="w-1.5 h-1.5 rounded-full flex-shrink-0"
        style={{
          background: taken
            ? "rgb(16, 185, 129)"
            : "rgb(var(--text-muted) / 0.4)",
        }}
        title={taken ? t("agents.tab_taken_tooltip") : t("agents.tab_untaken_tooltip")}
      />
      {agent.status === "config_broken" && (
        <AlertTriangle className="w-3 h-3 text-critical" />
      )}
    </button>
  );
}

// ──────────────────────────────────────────────────────────────────
// 渠道行
// ──────────────────────────────────────────────────────────────────
function ChannelRow({
  profile,
  agent,
  isCurrent,
  protocolMatch,
  onApply,
  onEdit,
}: {
  profile: ProviderProfile;
  agent: DiscoveredAgent;
  isCurrent: boolean;
  protocolMatch: boolean;
  onApply: () => void;
  onEdit: () => void;
}) {
  const host = (() => {
    try {
      return new URL(profile.base_url).host;
    } catch {
      return profile.base_url;
    }
  })();

  // 匹配 preset 拿品牌色 + SVG 图标
  const matchedPreset: ProviderPreset | undefined = PROVIDER_PRESETS.find(
    (p) => p.base_url === profile.base_url && p.protocol === profile.protocol,
  );

  const canApply =
    profile.enabled && profile.credential_set && !isCurrent && protocolMatch;

  return (
    <div
      className={cn(
        "group flex items-center gap-3 px-4 py-3 rounded-lg border bg-bg-elev transition-colors",
        isCurrent
          ? "border-accent/40 bg-accent/[0.03]"
          : "border-border-soft hover:border-text-muted/40",
        !profile.enabled && "opacity-55",
        !protocolMatch && "opacity-50",
      )}
    >
      {/* 头像 */}
      <BrandIcon
        preset={matchedPreset}
        name={profile.name}
        color={matchedPreset?.color}
        size={36}
        rounded="md"
      />

      {/* 主体 */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <h3 className="text-[13.5px] font-medium truncate">
            {profile.name}
          </h3>
          <span
            className="text-[9.5px] font-mono uppercase tracking-wider px-1.5 py-0.5 rounded bg-bg-elev2 text-text-muted"
            title={`协议：${profile.protocol}`}
          >
            {PROTOCOL_LABELS[profile.protocol] ?? profile.protocol}
          </span>
          {!protocolMatch && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-bg-elev2 text-text-muted">
              与 {agent.platform} 不兼容
            </span>
          )}
          {isCurrent && (
            <span className="inline-flex items-center gap-0.5 text-[10px] font-medium px-1.5 py-0.5 rounded text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border border-emerald-500/20">
              <CheckCircle2 className="w-2.5 h-2.5" />
              当前使用
            </span>
          )}
          {profile.is_default && !isCurrent && (
            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded text-accent bg-accent/10">
              默认
            </span>
          )}
          {!profile.credential_set && (
            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded text-amber-600 dark:text-amber-400 bg-amber-500/10">
              未设凭据
            </span>
          )}
          {!profile.enabled && (
            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded text-text-muted bg-bg-elev2">
              已停用
            </span>
          )}
        </div>
        <div className="text-[11.5px] text-accent font-mono truncate">
          {profile.base_url ? (
            <span title={profile.base_url}>
              {host || profile.base_url}
            </span>
          ) : (
            <span className="text-text-muted">未配置 base_url</span>
          )}
        </div>
      </div>

      {/* 操作 */}
      <div className="flex items-center gap-1.5 flex-shrink-0">
        {!profile.credential_set ? (
          <button
            onClick={onEdit}
            title="此渠道尚未设置 API 凭据，配置后即可一键启用托管"
            className="text-[11.5px] px-2.5 py-1 rounded border border-amber-500/40 text-amber-600 dark:text-amber-400 hover:bg-amber-500/10"
          >
            配置凭据后启用
          </button>
        ) : isCurrent ? (
          <span className="text-[11.5px] px-2.5 py-1 rounded text-text-muted">
            已应用
          </span>
        ) : (
          <button
            onClick={onApply}
            disabled={!canApply || agent.status === "candidate"}
            title={
              !protocolMatch
                ? `此渠道是 ${profile.protocol} 协议，与 ${agent.platform} Agent 不兼容`
                : agent.status === "candidate"
                  ? "候选 Agent 需先确认后才能启用渠道"
                  : "改写此 Agent 的配置文件，将 base_url 指向 ClawHeart 反代"
            }
            className="text-[11.5px] px-3 py-1 rounded bg-accent text-white hover:bg-accent/90 disabled:opacity-40 disabled:cursor-not-allowed"
          >
            启用
          </button>
        )}
        <button
          onClick={onEdit}
          className="w-7 h-7 flex items-center justify-center rounded text-text-muted hover:text-text hover:bg-bg-elev2 opacity-0 group-hover:opacity-100 transition-opacity"
          title="编辑渠道"
        >
          <Settings className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 空渠道占位
// ──────────────────────────────────────────────────────────────────
function EmptyChannels({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="px-6 py-12 text-center border border-dashed border-border-soft rounded-lg">
      <div className="text-[13px] text-text-muted mb-3">
        尚未配置任何模型渠道
      </div>
      <button
        onClick={onCreate}
        className="flex items-center gap-1.5 mx-auto px-3 py-1.5 text-[12px] rounded-md bg-accent text-white hover:bg-accent/90"
      >
        <Plus className="w-3.5 h-3.5" /> 新建第一个渠道
      </button>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 候选 Agent 行（confirm/ignore）
// ──────────────────────────────────────────────────────────────────
function CandidateRow({ agent }: { agent: DiscoveredAgent }) {
  const confirmMutation = useConfirmUnknownAgent();
  const ignoreMutation = useIgnoreUnknownAgent();
  const busy = confirmMutation.isPending || ignoreMutation.isPending;

  return (
    <div className="flex items-center gap-3 px-3 py-2.5 rounded-md border border-amber-500/20 bg-amber-500/[0.04]">
      <div className="w-7 h-7 rounded flex items-center justify-center flex-shrink-0 text-amber-600 dark:text-amber-400 bg-amber-500/10">
        <ScanSearch className="w-3.5 h-3.5" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-0.5 flex-wrap">
          <span className="text-[12.5px] font-medium">{agent.agent_name}</span>
          <span className="text-[10.5px] text-text-muted font-mono">
            {agent.platform.startsWith("unknown:")
              ? agent.platform.slice(8)
              : agent.platform}
          </span>
        </div>
        <div className="text-[10.5px] text-text-muted font-mono truncate">
          {agent.config_path}
          {agent.discovery_signals && agent.discovery_signals.length > 0 && (
            <span className="ml-1.5">
              · 线索：{agent.discovery_signals.slice(0, 3).join(", ")}
              {agent.discovery_signals.length > 3 && " …"}
            </span>
          )}
        </div>
      </div>
      <div className="flex items-center gap-1.5 flex-shrink-0">
        <button
          onClick={() => confirmMutation.mutate(agent.platform)}
          disabled={busy}
          className="flex items-center gap-1 text-[11px] px-2 py-1 rounded bg-accent text-white hover:bg-accent/90 disabled:opacity-50"
        >
          {confirmMutation.isPending ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <ShieldCheck className="w-3 h-3" />
          )}
          纳入
        </button>
        <button
          onClick={() => ignoreMutation.mutate(agent.platform)}
          disabled={busy}
          className="flex items-center gap-1 text-[11px] px-2 py-1 rounded text-text-muted hover:text-text border border-border hover:border-text-muted disabled:opacity-50"
        >
          {ignoreMutation.isPending ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : (
            <X className="w-3 h-3" />
          )}
          忽略
        </button>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 计算每个 Agent 当前生效的模型渠道名（扫最近 10 个 batch）
// ──────────────────────────────────────────────────────────────────
function useAgentCurrentChannels(): Map<string, string> {
  const { data: batches = [] } = useApplyBatches();
  const { data: profiles = [] } = useProviderProfiles();
  const recent = batches.slice(0, 10);

  const snapshotQueries = useQueries({
    queries: recent.map((b) => ({
      queryKey: ["agent_config", "batch_snapshots", b.batch_id],
      queryFn: () => listBatchSnapshots(b.batch_id),
      staleTime: 60 * 1000,
    })),
  });

  return useMemo(() => {
    const map = new Map<string, string>();
    for (let i = 0; i < recent.length; i++) {
      const snapshots: SnapshotDto[] = snapshotQueries[i]?.data ?? [];
      const profileId = recent[i].profile_id;
      if (!profileId) continue;
      const profile = profiles.find((p) => p.id === profileId);
      if (!profile) continue;
      for (const s of snapshots) {
        if (s.rolled_back_at) continue;
        if (!map.has(s.agent_id)) {
          map.set(s.agent_id, profile.name);
        }
      }
    }
    return map;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    profiles,
    JSON.stringify(snapshotQueries.map((q) => q.data?.length ?? 0)),
    recent.length,
  ]);
}

// ──────────────────────────────────────────────────────────────────
// 已分配渠道列表 —— 集成托管工具栏 + 渠道列表
// ──────────────────────────────────────────────────────────────────
function AssignedChannelList({
  profiles,
  agent,
  currentChannelName,
  isProtocolCompatible,
  onSelectFromLibrary,
  onCreateNew,
  onOpenHistory,
  onOpenImport,
  onApply,
  onEdit,
}: {
  profiles: ProviderProfile[];
  agent: DiscoveredAgent;
  currentChannelName: string | null;
  isProtocolCompatible: (p: ProviderProfile) => boolean;
  onSelectFromLibrary: () => void;
  onCreateNew: () => void;
  onOpenHistory: () => void;
  onOpenImport: () => void;
  onApply: (id: string) => void;
  onEdit: (id: string) => void;
}) {
  // 已分配内按 当前生效 → 已设凭据 → 未设凭据 排序
  const sorted = [...profiles].sort((a, b) => {
    const aCur = a.name === currentChannelName ? 0 : 1;
    const bCur = b.name === currentChannelName ? 0 : 1;
    if (aCur !== bCur) return aCur - bCur;
    const aReady = a.credential_set ? 0 : 1;
    const bReady = b.credential_set ? 0 : 1;
    return aReady - bReady;
  });

  const expectedProto = (() => {
    if (agent.platform === "claude") return "ANTHROPIC";
    if (agent.platform === "codex") return "OPENAI";
    if (agent.platform === "gemini") return "GEMINI";
    if (agent.platform === "cursor" || agent.platform === "windsurf")
      return "OpenAI / Anthropic";
    return "通用";
  })();

  return (
    <>
      {/* 工具栏：信息行 + 操作行 */}
      <AgentToolbar
        agent={agent}
        currentChannelName={currentChannelName}
        expectedProto={expectedProto}
        assignedCount={sorted.length}
        onSelectFromLibrary={onSelectFromLibrary}
        onCreateNew={onCreateNew}
        onOpenHistory={onOpenHistory}
        onOpenImport={onOpenImport}
      />

      {/* 已分配列表 */}
      {sorted.length === 0 ? (
        <div className="px-6 py-10 text-center border border-dashed border-border-soft rounded-md">
          <div className="text-[13px] text-text-muted mb-1">
            尚未为 {agent.agent_name} 分配任何渠道
          </div>
          <div className="text-[11.5px] text-text-muted">
            点击上方「从渠道库选择」或「新建」按钮添加
          </div>
        </div>
      ) : (
        <div className="space-y-2">
          {sorted.map((p) => (
            <ChannelRow
              key={p.id}
              profile={p}
              agent={agent}
              isCurrent={p.name === currentChannelName}
              protocolMatch={isProtocolCompatible(p)}
              onApply={() => onApply(p.id)}
              onEdit={() => onEdit(p.id)}
            />
          ))}
        </div>
      )}
    </>
  );
}

// ──────────────────────────────────────────────────────────────────
// Agent 工具栏（信息 + 托管开关 + 操作）
// ──────────────────────────────────────────────────────────────────
// 支持反向导入的平台白名单（与后端 IMPORTABLE_PLATFORMS 一致；不支持的也显示按钮但 Dialog 内提示）
const IMPORTABLE_PLATFORMS = new Set([
  "openclaw",
  "openeva",
  "opencode",
  "hermes",
  "claude",
  "codex",
  "gemini",
]);

function AgentToolbar({
  agent,
  currentChannelName,
  expectedProto,
  assignedCount,
  onSelectFromLibrary,
  onCreateNew,
  onOpenHistory,
  onOpenImport,
}: {
  agent: DiscoveredAgent;
  currentChannelName: string | null;
  expectedProto: string;
  assignedCount: number;
  onSelectFromLibrary: () => void;
  onCreateNew: () => void;
  onOpenHistory: () => void;
  onOpenImport: () => void;
}) {
  const { t } = useTranslation();
  const agentId = `${agent.platform}/${agent.agent_name}`;
  const { data: batches = [] } = useApplyBatches();
  const { data: applyReal } = useApplyRealStatus();
  const rollbackSnapshot = useRollbackSnapshot();

  const recent = batches.slice(0, 10);
  const snapshotQueries = useQueries({
    queries: recent.map((b) => ({
      queryKey: ["agent_config", "batch_snapshots", b.batch_id],
      queryFn: () => listBatchSnapshots(b.batch_id),
      staleTime: 60 * 1000,
    })),
  });

  // 找当前未回滚的 snapshot；统计该 agent 的总变更数
  const { activeSnapshot, totalChanges } = useMemo(() => {
    let active: SnapshotDto | null = null;
    let total = 0;
    for (let i = 0; i < recent.length; i++) {
      const ss: SnapshotDto[] = snapshotQueries[i]?.data ?? [];
      for (const s of ss) {
        if (s.agent_id !== agentId) continue;
        total += 1;
        if (!active && !s.rolled_back_at) {
          active = s;
        }
      }
    }
    return { activeSnapshot: active, totalChanges: total };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    agentId,
    JSON.stringify(snapshotQueries.map((q) => q.data?.length ?? 0)),
    recent.length,
  ]);

  const taken = !!activeSnapshot;
  const realEnabled = applyReal?.enabled ?? false;

  async function handleToggle() {
    if (!taken) {
      // 当前未托管：引导用户去启用某个渠道
      if (assignedCount === 0) {
        toast.info("请先分配或新建一个兼容渠道", {
          description: "点击右侧「从渠道库选择」或「新建」按钮",
        });
        return;
      }
      toast.info("请点击下方渠道的「启用」按钮选择具体渠道", {
        description: "ClawHeart 会用所选渠道的凭据改写 Agent 配置",
      });
      return;
    }
    // 已托管：confirm + rollback
    const hint = realEnabled
      ? "将真实还原 Agent 配置文件"
      : "Dry-run 模式：仅还原沙箱内容";
    if (!confirm(`关闭托管？\n${hint}`)) return;
    try {
      await rollbackSnapshot.mutateAsync({
        snapshotId: activeSnapshot!.id,
        dryRun: !realEnabled,
      });
      toast.success("已关闭托管，配置文件已还原");
    } catch (e) {
      toast.error(`关闭失败：${e}`);
    }
  }

  // 静默使用 expectedProto / currentChannelName / assignedCount —— 简化版工具栏不展示
  void expectedProto;
  void currentChannelName;
  void assignedCount;

  return (
    <div className="mb-3 flex items-center gap-2 flex-wrap">
      {/* 托管开关 */}
      <TakeoverToggle
        taken={taken}
        loading={rollbackSnapshot.isPending}
        onClick={handleToggle}
      />

      {/* 操作按钮组 */}
      <button
        onClick={onSelectFromLibrary}
        className="flex items-center gap-1 px-2.5 py-1 text-[11.5px] rounded border border-border hover:border-text-muted text-text-dim bg-bg-elev"
      >
        <ChevronRight className="w-3 h-3" />
        {t("agents.toolbar_select_from_library")}
      </button>
      <button
        onClick={onCreateNew}
        className="flex items-center gap-1 px-2.5 py-1 text-[11.5px] rounded border border-border hover:border-text-muted text-text-dim bg-bg-elev"
      >
        <Plus className="w-3 h-3" />
        {t("agents.toolbar_create_new")}
      </button>
      {IMPORTABLE_PLATFORMS.has(agent.platform) && (
        <button
          onClick={onOpenImport}
          className="flex items-center gap-1 px-2.5 py-1 text-[11.5px] rounded border border-accent/30 hover:border-accent text-accent bg-accent/5"
        >
          <Download className="w-3 h-3" />
          {t("agents.toolbar_import_from_config")}
        </button>
      )}
      <button
        onClick={onOpenHistory}
        disabled={totalChanges === 0}
        className="flex items-center gap-1 px-2.5 py-1 text-[11.5px] rounded border border-border hover:border-text-muted text-text-dim bg-bg-elev disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <History className="w-3 h-3" />
        {t("agents.toolbar_history")}
        {totalChanges > 0 && (
          <span className="text-text-muted">{totalChanges}</span>
        )}
      </button>

      {!realEnabled && taken && (
        <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400">
          {t("takeover.dry_run")}
        </span>
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 托管开关（iOS 风格 toggle）
// ──────────────────────────────────────────────────────────────────
function TakeoverToggle({
  taken,
  loading,
  onClick,
}: {
  taken: boolean;
  loading: boolean;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  return (
    <button
      onClick={onClick}
      disabled={loading}
      className={cn(
        "flex items-center gap-2 px-2 py-1 rounded-md text-[12px] font-medium transition-colors disabled:opacity-50",
        taken
          ? "text-emerald-700 dark:text-emerald-400"
          : "text-text-dim hover:text-text",
      )}
      title={taken ? "点击关闭托管（还原配置文件）" : "点击下方渠道的「启用」开启托管"}
    >
      {/* Toggle visual */}
      <span
        className={cn(
          "relative inline-block w-9 h-5 rounded-full transition-colors flex-shrink-0",
          taken ? "bg-emerald-500" : "bg-bg-elev2 border border-border",
        )}
      >
        <span
          className={cn(
            "absolute top-0.5 w-4 h-4 rounded-full bg-white shadow-sm transition-all",
            taken ? "left-[18px]" : "left-0.5",
          )}
        />
      </span>
      <span>
        {loading ? (
          <Loader2 className="w-3 h-3 animate-spin inline" />
        ) : taken ? (
          <>
            <Zap className="w-3 h-3 inline -mt-0.5 mr-0.5" />
            {t("takeover.managed")}
          </>
        ) : (
          t("takeover.unmanaged")
        )}
      </span>
    </button>
  );
}
