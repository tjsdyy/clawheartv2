/**
 * 「管理分配」弹层（渠道库视角）—— 一个渠道分配给哪些 Agent
 *
 * 与 SelectChannelDialog 对称：那个是 Agent 视角选渠道，这个是渠道视角选 Agent。
 *
 * 通过 useAllAssignments 找出该渠道当前已分配给哪些 Agent，复选框勾选，
 * 一次性 assign/unassign 多个 Agent。
 */
import { useEffect, useMemo, useState } from "react";
import { X, Save, Loader2, AlertTriangle } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { useAgents, type DiscoveredAgent } from "@/hooks/useAgents";
import {
  useAllAssignments,
  useAssignChannel,
  useUnassignChannel,
} from "@/hooks/useChannelAssignments";
import type { ProviderProfile } from "@/hooks/useProviders";

interface Props {
  profile: ProviderProfile;
  onClose: () => void;
}

const PLATFORM_LABELS: Record<string, string> = {
  claude: "Claude",
  codex: "Codex",
  cursor: "Cursor",
  gemini: "Gemini",
  windsurf: "Windsurf",
  openclaw: "OpenClaw",
  openeva: "OpenEva",
};

function matchesPlatform(protocol: string, platform: string): boolean {
  if (platform === "claude") return protocol === "anthropic";
  if (platform === "codex") return protocol === "openai" || protocol === "openai_responses";
  if (platform === "gemini") return protocol === "gemini";
  if (platform === "cursor" || platform === "windsurf") {
    return protocol === "openai" || protocol === "anthropic";
  }
  return true;
}

export function ManageAssignmentsDialog({ profile, onClose }: Props) {
  const { data: agents = [] } = useAgents();
  const { data: assignments = [] } = useAllAssignments();
  const assignMutation = useAssignChannel();
  const unassignMutation = useUnassignChannel();

  // 已分配给当前 profile 的 agent_id 列表
  const initialAssigned = useMemo(
    () =>
      assignments
        .filter((a) => a.profile_id === profile.id)
        .map((a) => a.agent_id),
    [assignments, profile.id],
  );

  const [selected, setSelected] = useState<Set<string>>(
    new Set(initialAssigned),
  );

  useEffect(() => {
    setSelected(new Set(initialAssigned));
  }, [initialAssigned.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  // 过滤掉候选 Agent
  const managed = agents.filter((a) => a.status !== "candidate");
  const compatible = managed.filter((a) => matchesPlatform(profile.protocol, a.platform));
  const incompatible = managed.filter((a) => !matchesPlatform(profile.protocol, a.platform));

  function toggle(agentId: string) {
    const next = new Set(selected);
    if (next.has(agentId)) next.delete(agentId);
    else next.add(agentId);
    setSelected(next);
  }

  async function handleSave() {
    const toAdd: string[] = [];
    const toRemove: string[] = [];
    for (const a of managed) {
      const id = `${a.platform}/${a.agent_name}`;
      const was = initialAssigned.includes(id);
      const now = selected.has(id);
      if (now && !was) toAdd.push(id);
      if (!now && was) toRemove.push(id);
    }

    try {
      for (const id of toAdd) {
        await assignMutation.mutateAsync({
          agentId: id,
          profileId: profile.id,
        });
      }
      for (const id of toRemove) {
        await unassignMutation.mutateAsync({
          agentId: id,
          profileId: profile.id,
        });
      }
      const summary = [];
      if (toAdd.length > 0) summary.push(`+${toAdd.length}`);
      if (toRemove.length > 0) summary.push(`-${toRemove.length}`);
      toast.success(
        summary.length > 0
          ? `已更新「${profile.name}」分配 (${summary.join(" ")})`
          : `「${profile.name}」分配未变化`,
      );
      onClose();
    } catch (e) {
      console.error(e);
    }
  }

  const submitting =
    assignMutation.isPending || unassignMutation.isPending;

  return (
    <div
      className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl max-h-[80vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div>
            <h3 className="text-[14px] font-semibold tracking-tight">
              管理分配 · {profile.name}
            </h3>
            <div className="text-[11.5px] text-text-muted mt-0.5 font-mono">
              {profile.protocol} · 已选 {selected.size} 个 Agent
            </div>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Body */}
        <div className="flex-1 overflow-auto p-5 space-y-4">
          {managed.length === 0 ? (
            <div className="px-4 py-8 text-center text-[12.5px] text-text-muted border border-dashed border-border-soft rounded-md">
              本机尚未发现已纳入管理的 Agent。
              <br />
              先在「Agent 发现」页确认候选 Agent。
            </div>
          ) : (
            <>
              {/* 兼容的 Agent */}
              <section>
                <div className="text-[10.5px] uppercase tracking-wider text-text-muted mb-2 font-mono">
                  兼容此渠道协议 ({compatible.length})
                </div>
                {compatible.length === 0 ? (
                  <div className="text-[12px] text-text-muted px-3 py-3 border border-dashed border-border-soft rounded-md">
                    没有兼容 {profile.protocol} 协议的 Agent
                  </div>
                ) : (
                  <div className="space-y-1.5">
                    {compatible.map((a) => (
                      <AgentItem
                        key={`${a.platform}/${a.agent_name}`}
                        agent={a}
                        checked={selected.has(`${a.platform}/${a.agent_name}`)}
                        onToggle={() => toggle(`${a.platform}/${a.agent_name}`)}
                        disabled={false}
                      />
                    ))}
                  </div>
                )}
              </section>

              {/* 不兼容的（仅展示） */}
              {incompatible.length > 0 && (
                <section>
                  <div className="text-[10.5px] uppercase tracking-wider text-text-muted mb-2 font-mono flex items-center gap-1">
                    <AlertTriangle className="w-3 h-3" />
                    其他协议 Agent ({incompatible.length}) · 不可分配
                  </div>
                  <div className="space-y-1.5 opacity-50">
                    {incompatible.map((a) => (
                      <AgentItem
                        key={`${a.platform}/${a.agent_name}`}
                        agent={a}
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
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center justify-end gap-2">
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={handleSave}
            disabled={submitting}
            className="flex items-center gap-1.5 px-4 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
          >
            {submitting ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Save className="w-3.5 h-3.5" />
            )}
            保存
          </button>
        </footer>
      </div>
    </div>
  );
}

function AgentItem({
  agent,
  checked,
  onToggle,
  disabled,
}: {
  agent: DiscoveredAgent;
  checked: boolean;
  onToggle: () => void;
  disabled: boolean;
}) {
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
      <div
        className="w-7 h-7 rounded flex items-center justify-center text-[10px] font-bold font-mono flex-shrink-0"
        style={{
          background: "rgb(var(--bg-elev2))",
          color: "rgb(var(--text-muted))",
        }}
      >
        {agent.agent_name.charAt(0)}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-0.5">
          <span className="text-[12.5px] font-medium truncate">
            {agent.agent_name}
          </span>
          <span className="text-[10.5px] font-mono text-text-muted">
            {PLATFORM_LABELS[agent.platform] ?? agent.platform}
          </span>
        </div>
        <div className="text-[10.5px] text-text-muted font-mono truncate">
          {agent.config_path ?? "无配置路径"}
        </div>
      </div>
    </label>
  );
}
