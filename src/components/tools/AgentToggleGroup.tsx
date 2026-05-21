import { Loader2, ArrowRight } from "lucide-react";
import {
  useMoveToSsot,
  useToggleSkillBinding,
  useRepairBinding,
  isBindingEnabled,
  bindingWarning,
  type AgentBinding,
} from "@/hooks/useSkills";

/** 常用 Agent 预设；即使尚未发现也显示空槽，方便启用 */
export const AGENT_PRESETS = ["claude", "openeva", "codex", "cursor", "openclaw"];

/** 取 Agent 名首字母大写 */
export function agentLetter(name: string): string {
  return name.charAt(0).toUpperCase();
}

/**
 * Per-Agent 启用状态 + 切换按钮组。
 * - 绿色 = 已链入集中库（已启用）· 点击移除
 * - 灰色 = 未启用 · 点击创建 symlink
 * - 黄色 = unmanaged / external symlink · 点击 repair（或提示先迁入）
 * - 红色 = broken symlink · 点击 repair
 *
 * 只有 in_ssot 的 skill 才能 toggle；unmanaged 显示禁用态。
 */
export function AgentToggleGroup({
  skillId,
  inSsot,
  bindings,
}: {
  skillId: string;
  inSsot: boolean;
  bindings: AgentBinding[];
}) {
  const toggle = useToggleSkillBinding();
  const repair = useRepairBinding();

  const bindingMap = new Map(bindings.map((b) => [b.agent_name, b]));
  const allAgents = Array.from(
    new Set([...AGENT_PRESETS, ...bindings.map((b) => b.agent_name)]),
  );

  return (
    <div className="flex items-center gap-1 flex-wrap">
      {allAgents.map((agent) => {
        const b = bindingMap.get(agent);
        const enabled = b ? isBindingEnabled(b) : false;
        const warn = b ? bindingWarning(b) : null;
        const letter = agentLetter(agent);

        let bg = "rgb(var(--bg-elev2))";
        let fg = "rgb(var(--text-muted))";
        let border = "rgb(var(--border-soft))";
        let title = `.${agent} · 未启用 · 点击启用（已链入集中库）`;

        if (enabled) {
          bg = "color-mix(in srgb, rgb(var(--accent)) 14%, transparent)";
          fg = "rgb(var(--accent))";
          border = "color-mix(in srgb, rgb(var(--accent)) 40%, transparent)";
          title = `.${agent} · 已启用（已链入集中库）· 点击移除`;
        } else if (warn === "unmanaged") {
          bg = "rgb(245 158 11 / 0.10)";
          fg = "rgb(245 158 11)";
          border = "rgb(245 158 11 / 0.35)";
          title = `.${agent} · 真实文件未纳入集中库管理 · 点「迁入集中库」`;
        } else if (warn === "external") {
          bg = "rgb(245 158 11 / 0.10)";
          fg = "rgb(245 158 11)";
          border = "rgb(245 158 11 / 0.35)";
          title = `.${agent} · symlink 指向集中库之外 · 点击修复`;
        } else if (warn === "broken") {
          bg = "rgb(var(--critical) / 0.10)";
          fg = "rgb(var(--critical))";
          border = "rgb(var(--critical) / 0.35)";
          title = `.${agent} · symlink 已损坏 · 点击修复`;
        }

        const canToggle = inSsot && (enabled || !b);
        const canRepair = inSsot && (warn === "external" || warn === "broken");
        const disabled =
          (!canToggle && !canRepair) || toggle.isPending || repair.isPending;

        function handleClick(e: React.MouseEvent) {
          e.preventDefault();
          e.stopPropagation();
          if (canRepair) {
            repair.mutate({ id: skillId, agent });
            return;
          }
          if (canToggle) {
            toggle.mutate({ id: skillId, agent, enabled: !enabled });
          }
        }

        return (
          <button
            key={agent}
            disabled={disabled}
            onClick={handleClick}
            title={title}
            className="inline-flex items-center justify-center w-5 h-5 rounded text-[10px] font-mono font-bold uppercase border transition-transform hover:scale-110 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
            style={{ background: bg, color: fg, borderColor: border }}
          >
            {letter}
          </button>
        );
      })}
    </div>
  );
}

/** Unmanaged skill 行内"迁入集中库"按钮 */
export function MoveToSsotInline({ skillId }: { skillId: string }) {
  const move = useMoveToSsot();
  return (
    <button
      onClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        move.mutate(skillId);
      }}
      disabled={move.isPending}
      className="mt-1.5 inline-flex items-center gap-1 text-[10.5px] px-2 py-0.5 rounded border border-amber-500/40 text-amber-500 hover:bg-amber-500/10 disabled:opacity-50"
    >
      {move.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <ArrowRight className="w-3 h-3" />}
      迁入集中库
    </button>
  );
}
