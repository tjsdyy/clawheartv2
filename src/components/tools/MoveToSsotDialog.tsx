import { X, ArrowRight, Archive, FolderInput, Loader2 } from "lucide-react";
import type { DiscoveredSkill } from "@/hooks/useSkills";

interface Props {
  skills: DiscoveredSkill[];
  ssotPath: string;
  pending: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function MoveToSsotDialog({
  skills,
  ssotPath,
  pending,
  onConfirm,
  onCancel,
}: Props) {
  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-xl bg-bg rounded-xl shadow-2xl border border-border overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            <FolderInput className="w-4 h-4 text-accent" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              迁入集中库 · {skills.length} 个技能
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

        <div className="px-5 py-4 max-h-[55vh] overflow-y-auto space-y-3">
          <p className="text-[12.5px] text-text-dim leading-relaxed">
            将以下散落技能迁入主目录{" "}
            <code className="font-mono text-[11.5px] text-text">{ssotPath}</code>
            。原位置会改成 symlink，其他 Agent 也能立刻共享。
          </p>

          <div className="rounded-md border border-border-soft divide-y divide-border-soft text-[12px] font-mono">
            {skills.slice(0, 10).map((s) => (
              <div
                key={s.id}
                className="flex items-center gap-2 px-3 py-1.5"
                title={s.source_path}
              >
                <span className="text-text">{s.name}</span>
                {s.version && (
                  <span className="text-text-muted">v{s.version}</span>
                )}
                <ArrowRight className="w-3 h-3 text-text-muted ml-auto" />
                <code className="text-[10.5px] text-text-dim truncate max-w-[280px]">
                  {ssotPath}/{s.id}
                </code>
              </div>
            ))}
            {skills.length > 10 && (
              <div className="px-3 py-1.5 text-text-muted text-[11.5px]">
                …还有 {skills.length - 10} 个
              </div>
            )}
          </div>

          <div className="rounded-md bg-bg-elev/40 border border-border-soft px-3 py-2.5 text-[11.5px] text-text-dim leading-relaxed">
            <div className="flex items-center gap-1.5 mb-1 text-text">
              <Archive className="w-3 h-3" />
              <span className="font-semibold">自动备份</span>
            </div>
            原始内容会先 zip 备份到{" "}
            <code className="font-mono text-text-muted">
              ~/.clawheart-v2/auto-backups/skills/
            </code>
            ，回滚有据。
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
            className="px-3.5 py-1.5 rounded-md text-[12.5px] font-medium bg-accent text-white hover:bg-accent/90 disabled:opacity-50 flex items-center gap-1.5"
          >
            {pending ? (
              <>
                <Loader2 className="w-3 h-3 animate-spin" />
                迁入中…
              </>
            ) : (
              <>
                <ArrowRight className="w-3 h-3" />
                确认迁入 {skills.length} 项
              </>
            )}
          </button>
        </footer>
      </div>
    </div>
  );
}
