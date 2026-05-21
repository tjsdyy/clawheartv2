/**
 * 模型渠道库（重新定位）
 *
 * 职责：全局渠道的 CRUD + 凭据 + 跨 Agent 分配视图
 * 不负责：启用/托管/历史（那是 AgentsTool 的事）
 *
 * 结构：
 * ┌─ 渠道库 · 12 ↻  ⊕ 新建 ─────────────────────┐
 * ├──────────────────────────────────────────────┤
 * │ ● Kimi          [ANTHROPIC] [默认]           │
 * │   api.moonshot.cn                            │
 * │   分配给：Claude, Cursor   [⚙][🗑]          │
 * ├──────────────────────────────────────────────┤
 * │ ● DeepSeek      [ANTHROPIC]                  │
 * │   api.deepseek.com                           │
 * │   分配给：Claude          [⚙][🗑]            │
 * ├──────────────────────────────────────────────┤
 * │ ● Qwen-Coder    [OPENAI] [未分配]            │
 * │   dashscope.aliyuncs.com                     │
 * │   未分配给任何 Agent       [⚙][🗑]           │
 * └──────────────────────────────────────────────┘
 */
import { useMemo, useState } from "react";
import {
  Plus,
  RefreshCw,
  Pencil,
  Trash2,
  Loader2,
  Sparkles,
  Users as UsersIcon,
  Search,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import {
  useProviderProfiles,
  useDeleteProvider,
  useSetDefaultProvider,
  type ProviderProfile,
} from "@/hooks/useProviders";
import { useAllAssignments } from "@/hooks/useChannelAssignments";
import { useAgents } from "@/hooks/useAgents";
import { AddChannelDialog } from "@/components/agents/AddChannelDialog";
import { ManageAssignmentsDialog } from "@/components/agents/ManageAssignmentsDialog";
import { BrandIcon } from "@/components/agents/BrandIcon";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { PROVIDER_PRESETS } from "@/data/provider-presets";

const PROTOCOL_LABELS: Record<string, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  openai_responses: "OpenAI Resp.",
  gemini: "Gemini",
  ollama: "Ollama",
};

const PLATFORM_LABELS: Record<string, string> = {
  claude: "Claude",
  codex: "Codex",
  cursor: "Cursor",
  gemini: "Gemini",
  windsurf: "Windsurf",
  openclaw: "OpenClaw",
  openeva: "OpenEva",
};

export function ProvidersTool() {
  const { data: profiles = [], isLoading, refetch } = useProviderProfiles();
  const { data: assignments = [] } = useAllAssignments();
  const { data: agents = [] } = useAgents();
  const deleteProvider = useDeleteProvider();
  const setDefault = useSetDefaultProvider();

  const [addOpen, setAddOpen] = useState(false);
  const [editingProfileId, setEditingProfileId] = useState<string | null>(null);
  const [assignProfileId, setAssignProfileId] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [confirmDelete, setConfirmDelete] = useState<ProviderProfile | null>(null);

  const editingProfile = editingProfileId
    ? profiles.find((p) => p.id === editingProfileId) ?? null
    : null;
  const assignProfile = assignProfileId
    ? profiles.find((p) => p.id === assignProfileId) ?? null
    : null;

  // profile_id → 已分配的 agent_id 列表
  const assignmentsByProfile = useMemo(() => {
    const map = new Map<string, string[]>();
    for (const a of assignments) {
      const arr = map.get(a.profile_id) ?? [];
      arr.push(a.agent_id);
      map.set(a.profile_id, arr);
    }
    return map;
  }, [assignments]);

  const filtered = profiles.filter((p) => {
    if (!query.trim()) return true;
    const q = query.toLowerCase();
    return (
      p.name.toLowerCase().includes(q) ||
      p.base_url.toLowerCase().includes(q) ||
      p.protocol.toLowerCase().includes(q)
    );
  });

  const total = profiles.length;
  const assigned = profiles.filter((p) =>
    assignmentsByProfile.has(p.id),
  ).length;
  const unassigned = total - assigned;

  async function doDelete(profile: ProviderProfile) {
    try {
      await deleteProvider.mutateAsync(profile.id);
      toast.success(`已删除「${profile.name}」`);
      setConfirmDelete(null);
    } catch (e) {
      toast.error(`删除失败：${e}`);
    }
  }

  async function handleSetDefault(profile: ProviderProfile) {
    if (profile.is_default) return;
    await setDefault.mutateAsync(profile.id);
    toast.success(`「${profile.name}」已设为默认渠道`);
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <header className="px-6 py-3 border-b border-border-soft flex items-center gap-3 flex-shrink-0">
        <div>
          <h2 className="text-[16px] font-semibold">模型渠道库</h2>
          <p className="text-[11.5px] text-text-dim font-mono mt-0.5">
            全部 {total} · 已分配 {assigned} · 未分配 {unassigned}
          </p>
        </div>
        <div className="flex-1" />
        <div className="relative">
          <Search className="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索名称 / URL / 协议"
            className="pl-7 pr-2 py-1 text-[11.5px] bg-bg-elev border border-border rounded-md focus:border-accent outline-none w-48"
          />
        </div>
        <button
          onClick={() => refetch()}
          className="btn-ghost text-[12px]"
          title="刷新"
        >
          <RefreshCw className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => setAddOpen(true)}
          className="flex items-center gap-1 px-3 py-1.5 rounded-md bg-accent text-white text-[12px] font-medium hover:bg-accent/90"
        >
          <Plus className="w-3.5 h-3.5" />
          新建渠道
        </button>
      </header>

      {/* Body */}
      <div className="flex-1 overflow-auto p-6">
        {isLoading ? (
          <div className="text-text-muted text-[13px] text-center py-12">
            加载中…
          </div>
        ) : profiles.length === 0 ? (
          <EmptyLibrary onCreate={() => setAddOpen(true)} />
        ) : filtered.length === 0 ? (
          <div className="text-[13px] text-text-muted text-center py-12">
            没有匹配「{query}」的渠道
          </div>
        ) : (
          <div className="space-y-2 max-w-3xl">
            {filtered.map((p) => (
              <ChannelLibraryRow
                key={p.id}
                profile={p}
                agentIds={assignmentsByProfile.get(p.id) ?? []}
                agents={agents}
                onEdit={() => setEditingProfileId(p.id)}
                onManageAssign={() => setAssignProfileId(p.id)}
                onDelete={() => setConfirmDelete(p)}
                onSetDefault={() => handleSetDefault(p)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Dialogs */}
      {addOpen && (
        <AddChannelDialog onClose={() => setAddOpen(false)} />
      )}
      {editingProfile && (
        <AddChannelDialog
          onClose={() => setEditingProfileId(null)}
          editingProfile={editingProfile}
        />
      )}
      {assignProfile && (
        <ManageAssignmentsDialog
          profile={assignProfile}
          onClose={() => setAssignProfileId(null)}
        />
      )}
      {confirmDelete && (
        <ConfirmDialog
          title={`删除渠道「${confirmDelete.name}」？`}
          message={
            "此操作不可恢复。\n" +
            "Keychain 中的 API 凭据将一并移除，分配关系也会全部清除。"
          }
          confirmText="删除"
          dangerous
          loading={deleteProvider.isPending}
          onConfirm={() => doDelete(confirmDelete)}
          onCancel={() => setConfirmDelete(null)}
        />
      )}
    </div>
  );
}

function ChannelLibraryRow({
  profile,
  agentIds,
  agents,
  onEdit,
  onManageAssign,
  onDelete,
  onSetDefault,
}: {
  profile: ProviderProfile;
  agentIds: string[];
  agents: ReturnType<typeof useAgents>["data"];
  onEdit: () => void;
  onManageAssign: () => void;
  onDelete: () => void;
  onSetDefault: () => void;
}) {
  const host = (() => {
    try {
      return new URL(profile.base_url).host;
    } catch {
      return profile.base_url;
    }
  })();

  const matchedPreset = PROVIDER_PRESETS.find(
    (p) => p.base_url === profile.base_url && p.protocol === profile.protocol,
  );

  // 解析 agent_id → 友好名（platform/agent_name → platform 标签）
  const agentLabels = agentIds.map((id) => {
    const found = (agents ?? []).find(
      (a) => `${a.platform}/${a.agent_name}` === id,
    );
    if (found) {
      return PLATFORM_LABELS[found.platform] ?? found.platform;
    }
    return id.split("/")[0];
  });

  return (
    <div
      className={cn(
        "group flex items-start gap-3 px-4 py-3 rounded-lg border bg-bg-elev transition-colors",
        profile.is_default
          ? "border-accent/40"
          : "border-border-soft hover:border-text-muted/40",
        !profile.enabled && "opacity-55",
      )}
    >
      <BrandIcon
        preset={matchedPreset}
        name={profile.name}
        color={matchedPreset?.color}
        size={36}
        rounded="md"
      />

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5 flex-wrap">
          <h3 className="text-[13.5px] font-medium truncate">{profile.name}</h3>
          <span className="text-[9.5px] font-mono uppercase tracking-wider px-1.5 py-0.5 rounded bg-bg-elev2 text-text-muted">
            {PROTOCOL_LABELS[profile.protocol] ?? profile.protocol}
          </span>
          {profile.is_default && (
            <span className="inline-flex items-center gap-0.5 text-[10px] font-medium px-1.5 py-0.5 rounded text-accent bg-accent/10">
              <Sparkles className="w-2.5 h-2.5" /> 默认
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
          {agentIds.length === 0 && (
            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded text-text-muted bg-bg-elev2">
              未分配
            </span>
          )}
        </div>
        <div className="text-[11.5px] text-accent font-mono mb-1.5 truncate">
          {host}
        </div>
        <div className="flex items-center gap-1.5 text-[11px]">
          <UsersIcon className="w-3 h-3 text-text-muted flex-shrink-0" />
          <span className="text-text-muted">分配给:</span>
          {agentIds.length === 0 ? (
            <span className="text-text-muted">未分配给任何 Agent</span>
          ) : (
            <div className="flex flex-wrap gap-1">
              {agentLabels.map((label, i) => (
                <span
                  key={`${label}-${i}`}
                  className="px-1.5 py-0.5 rounded bg-accent/10 text-accent text-[10.5px]"
                >
                  {label}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* 操作 */}
      <div className="flex items-center gap-1 flex-shrink-0 opacity-50 group-hover:opacity-100 transition-opacity">
        {!profile.is_default && profile.credential_set && (
          <button
            onClick={onSetDefault}
            className="text-[11px] px-2 py-1 rounded text-text-muted hover:text-accent hover:bg-accent/5"
            title="设为默认渠道"
          >
            设默认
          </button>
        )}
        <button
          onClick={onManageAssign}
          className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-border hover:border-text-muted text-text-dim"
          title="管理分配"
        >
          <UsersIcon className="w-3 h-3" />
          分配
        </button>
        <button
          onClick={onEdit}
          className="w-7 h-7 flex items-center justify-center rounded text-text-muted hover:text-text hover:bg-bg-elev2"
          title="编辑"
        >
          <Pencil className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={onDelete}
          className="w-7 h-7 flex items-center justify-center rounded text-text-muted hover:text-critical hover:bg-critical/5"
          title="删除"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}

function EmptyLibrary({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="max-w-md mx-auto mt-12 px-6 py-12 text-center border border-dashed border-border-soft rounded-lg">
      <div className="text-[18px] font-semibold mb-2">渠道库为空</div>
      <div className="text-[12.5px] text-text-muted mb-5 leading-relaxed">
        渠道是 ClawHeart 路由 Agent 流量的上游凭据。
        创建后可在「Agent 发现」分配给各 Agent 使用。
      </div>
      <button
        onClick={onCreate}
        className="flex items-center gap-1.5 mx-auto px-4 py-2 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90"
      >
        <Plus className="w-3.5 h-3.5" />
        创建第一个渠道
      </button>
    </div>
  );
}
