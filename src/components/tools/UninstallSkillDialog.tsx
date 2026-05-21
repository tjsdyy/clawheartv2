import { X, Trash2, AlertTriangle, Archive, Loader2 } from "lucide-react";
import { isBindingEnabled, type DiscoveredSkill } from "@/hooks/useSkills";

interface Props {
  skill: DiscoveredSkill;
  pending: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function UninstallSkillDialog({
  skill,
  pending,
  onConfirm,
  onCancel,
}: Props) {
  const enabledAgents = skill.bindings.filter(isBindingEnabled).map((b) => b.agent_name);

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-lg bg-bg rounded-xl shadow-2xl border border-critical/40 overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            <Trash2 className="w-4 h-4 text-critical" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              卸载 {skill.name}
            </h3>
          </div>
          <button
            onClick={onCancel}
            disabled={pending}
            className="text-text-muted hover:text-text disabled:opacity-40"
          >
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="px-5 py-4 space-y-3 text-[12.5px] text-text-dim leading-relaxed">
          <div className="flex items-start gap-2 text-critical">
            <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5" />
            <span>
              此操作将 <strong>不可恢复地</strong> 删除该技能在集中库的主副本，并清除
              {" "}{enabledAgents.length} 个 Agent 中指向集中库的 symlink。
            </span>
          </div>

          <ul className="ml-1 space-y-1 text-[12px] text-text font-mono">
            <li className="flex items-start gap-2">
              <span className="text-critical">✗</span>
              <span className="text-text-dim">删集中库主副本：</span>
              <code className="text-text-muted">{skill.ssot_path ?? "—"}</code>
            </li>
            {enabledAgents.length > 0 && (
              <li className="flex items-start gap-2">
                <span className="text-critical">✗</span>
                <span className="text-text-dim">清除 symlink：</span>
                <span className="text-text-muted">
                  {enabledAgents.map((a) => `.${a}`).join(", ")}
                </span>
              </li>
            )}
          </ul>

          <div className="rounded-md bg-bg-elev/40 border border-border-soft px-3 py-2.5 text-[11.5px]">
            <div className="flex items-center gap-1.5 mb-1 text-text">
              <Archive className="w-3 h-3" />
              <span className="font-semibold">操作前自动备份</span>
            </div>
            zip 备份会存到{" "}
            <code className="font-mono text-text-muted">
              ~/.clawheart-v2/auto-backups/skills/
            </code>
            ，需要时可手动恢复。
          </div>
        </div>

        <footer className="px-5 py-3 border-t border-border bg-bg-elev/30 flex items-center justify-end gap-2">
          <button
            onClick={onCancel}
            disabled={pending}
            className="px-3.5 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2 disabled:opacity-40"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            disabled={pending}
            className="px-3.5 py-1.5 rounded-md text-[12.5px] font-medium bg-critical text-white hover:bg-critical/90 disabled:opacity-50 flex items-center gap-1.5"
          >
            {pending ? (
              <>
                <Loader2 className="w-3 h-3 animate-spin" />
                卸载中…
              </>
            ) : (
              <>
                <Trash2 className="w-3 h-3" />
                确认卸载
              </>
            )}
          </button>
        </footer>
      </div>
    </div>
  );
}
