/**
 * 「从渠道库选择」弹层
 *
 * 显示所有协议兼容的渠道，按"已分配 / 未分配"分组，复选框选择，
 * 一次性保存 → replace_agent_channels（事务）。
 */
import { useEffect, useState } from "react";
import { X, Save, Loader2, Plus } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { useProviderProfiles, type ProviderProfile } from "@/hooks/useProviders";
import {
  useAgentChannels,
  useReplaceAgentChannels,
} from "@/hooks/useChannelAssignments";
import { BrandIcon } from "./BrandIcon";
import { PROVIDER_PRESETS } from "@/data/provider-presets";

interface Props {
  agentId: string;
  agentName: string;
  agentPlatform: string;
  isCompatible: (p: ProviderProfile) => boolean;
  onClose: () => void;
  onCreateNew?: () => void;
}

export function SelectChannelDialog({
  agentId,
  agentName,
  agentPlatform,
  isCompatible,
  onClose,
  onCreateNew,
}: Props) {
  const { data: profiles = [] } = useProviderProfiles();
  const { data: assigned = [] } = useAgentChannels(agentId);
  const replaceMutation = useReplaceAgentChannels();

  const [selected, setSelected] = useState<Set<string>>(new Set(assigned));

  // 当 assigned 加载完成时初始化 selected
  useEffect(() => {
    setSelected(new Set(assigned));
  }, [assigned.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  function toggle(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelected(next);
  }

  async function handleSave() {
    try {
      await replaceMutation.mutateAsync({
        agentId,
        profileIds: Array.from(selected),
      });
      toast.success(`已为「${agentName}」分配 ${selected.size} 个渠道`);
      onClose();
    } catch (e) {
      console.error(e);
    }
  }

  const compatible = profiles.filter((p) => isCompatible(p));
  const incompatible = profiles.filter((p) => !isCompatible(p));

  function findPreset(p: ProviderProfile) {
    return PROVIDER_PRESETS.find(
      (pr) => pr.base_url === p.base_url && pr.protocol === p.protocol,
    );
  }

  return (
    <div
      className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl max-h-[85vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div>
            <h3 className="text-[14px] font-semibold tracking-tight">
              为「{agentName}」选择模型渠道
            </h3>
            <div className="text-[11.5px] text-text-muted mt-0.5">
              {agentPlatform} · 已选 {selected.size} 个
            </div>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Body */}
        <div className="flex-1 overflow-auto p-5 space-y-4">
          {profiles.length === 0 ? (
            <EmptyState onCreateNew={onCreateNew} />
          ) : (
            <>
              {/* 兼容协议的渠道 */}
              <section>
                <div className="text-[10.5px] uppercase tracking-wider text-text-muted mb-2 font-mono">
                  兼容当前 Agent ({compatible.length})
                </div>
                {compatible.length === 0 ? (
                  <div className="px-4 py-6 text-center text-[12.5px] text-text-muted border border-dashed border-border-soft rounded-md">
                    没有兼容 {agentPlatform} 协议的渠道。
                    {onCreateNew && (
                      <button
                        onClick={onCreateNew}
                        className="ml-2 text-accent hover:underline"
                      >
                        + 新建一个
                      </button>
                    )}
                  </div>
                ) : (
                  <div className="space-y-1.5">
                    {compatible.map((p) => (
                      <ProfileItem
                        key={p.id}
                        profile={p}
                        preset={findPreset(p)}
                        checked={selected.has(p.id)}
                        onToggle={() => toggle(p.id)}
                        disabled={false}
                      />
                    ))}
                  </div>
                )}
              </section>

              {/* 不兼容的（仅展示，不可选） */}
              {incompatible.length > 0 && (
                <section>
                  <div className="text-[10.5px] uppercase tracking-wider text-text-muted mb-2 font-mono">
                    其他协议 ({incompatible.length}) · 不兼容，不可分配
                  </div>
                  <div className="space-y-1.5 opacity-50">
                    {incompatible.map((p) => (
                      <ProfileItem
                        key={p.id}
                        profile={p}
                        preset={findPreset(p)}
                        checked={false}
                        onToggle={() => {}}
                        disabled={true}
                      />
                    ))}
                  </div>
                </section>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center gap-2">
          {onCreateNew && (
            <button
              onClick={onCreateNew}
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
            >
              <Plus className="w-3.5 h-3.5" />
              新建渠道
            </button>
          )}
          <div className="flex-1" />
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={handleSave}
            disabled={replaceMutation.isPending}
            className="flex items-center gap-1.5 px-4 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
          >
            {replaceMutation.isPending ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Save className="w-3.5 h-3.5" />
            )}
            保存分配
          </button>
        </footer>
      </div>
    </div>
  );
}

function ProfileItem({
  profile,
  preset,
  checked,
  onToggle,
  disabled,
}: {
  profile: ProviderProfile;
  preset: ReturnType<typeof PROVIDER_PRESETS.find>;
  checked: boolean;
  onToggle: () => void;
  disabled: boolean;
}) {
  const host = (() => {
    try {
      return new URL(profile.base_url).host;
    } catch {
      return profile.base_url;
    }
  })();

  return (
    <label
      className={cn(
        "flex items-center gap-3 px-3 py-2 rounded-md border bg-bg-elev cursor-pointer transition-colors",
        checked
          ? "border-accent/40 bg-accent/[0.04]"
          : "border-border-soft hover:border-text-muted/60",
        disabled && "cursor-not-allowed",
      )}
    >
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={onToggle}
        className="w-3.5 h-3.5 accent-accent flex-shrink-0"
      />
      <BrandIcon
        preset={preset || undefined}
        name={profile.name}
        color={preset?.color}
        size={28}
        rounded="sm"
      />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-0.5">
          <span className="text-[12.5px] font-medium truncate">
            {profile.name}
          </span>
          <span className="text-[9.5px] font-mono uppercase tracking-wider px-1 py-0.5 rounded bg-bg-elev2 text-text-muted">
            {profile.protocol}
          </span>
          {profile.is_default && (
            <span className="text-[10px] px-1 py-0.5 rounded text-accent bg-accent/10">
              默认
            </span>
          )}
          {!profile.credential_set && (
            <span className="text-[10px] px-1 py-0.5 rounded text-amber-600 dark:text-amber-400 bg-amber-500/10">
              未设凭据
            </span>
          )}
        </div>
        <div className="text-[10.5px] text-text-muted font-mono truncate">
          {host}
        </div>
      </div>
    </label>
  );
}

function EmptyState({ onCreateNew }: { onCreateNew?: () => void }) {
  return (
    <div className="px-6 py-10 text-center border border-dashed border-border-soft rounded-md">
      <div className="text-[13px] text-text-muted mb-3">
        渠道库为空，请先新建一个模型渠道
      </div>
      {onCreateNew && (
        <button
          onClick={onCreateNew}
          className="flex items-center gap-1.5 mx-auto px-3 py-1.5 text-[12px] rounded-md bg-accent text-white hover:bg-accent/90"
        >
          <Plus className="w-3.5 h-3.5" />
          新建渠道
        </button>
      )}
    </div>
  );
}
