import { useState } from "react";
import { X, ShieldAlert, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface Props {
  onClose: () => void;
  onConfirm: () => Promise<void>;
  busy?: boolean;
}

/**
 * 开启"实际写入 Agent 配置文件"的二次确认对话框。
 * 用户必须勾选 4 项理解后才能确认。
 */
export function ApplyRealRiskDialog({ onClose, onConfirm, busy }: Props) {
  const [understood, setUnderstood] = useState({
    real_files: false,
    snapshot: false,
    rollback: false,
    irreversible_after_edit: false,
  });

  const allChecked = Object.values(understood).every(Boolean);

  return (
    <div className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-lg bg-bg rounded-xl shadow-2xl border border-border overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border bg-amber-500/5">
          <div className="flex items-center gap-2.5">
            <ShieldAlert className="w-5 h-5 text-amber-500" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              启用实际写入 — 风险确认
            </h3>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="px-5 py-5 space-y-3 max-h-[60vh] overflow-y-auto">
          <p className="text-[12.5px] text-text-dim leading-relaxed">
            默认情况下，ClawHeart 的「自动应用到 Agent」只写入 dry-run 沙箱目录
            （<code className="font-mono text-[11px]">~/.clawheart-v2/dry-run/</code>），
            不会修改你电脑上 Cursor / Claude Code / Continue / OpenClaw 等真实配置文件。
          </p>

          <p className="text-[12.5px] text-text leading-relaxed">
            <strong>开启此选项后，</strong>「自动应用」会直接修改真实的 Agent 配置文件。
            请逐项确认你已理解以下事项：
          </p>

          <div className="space-y-2 pt-1">
            <CheckRow
              checked={understood.real_files}
              onChange={(v) => setUnderstood((u) => ({ ...u, real_files: v }))}
              label="我理解开启后 ClawHeart 会直接修改 Agent 的真实配置文件"
              desc="影响的目录包括 ~/.claude/、~/.continue/、Cursor User settings 等"
            />
            <CheckRow
              checked={understood.snapshot}
              onChange={(v) => setUnderstood((u) => ({ ...u, snapshot: v }))}
              label="我理解每次写入都会自动创建 snapshot 备份"
              desc="所有变更都保留 before_value，可在应用记录页随时回滚"
            />
            <CheckRow
              checked={understood.rollback}
              onChange={(v) => setUnderstood((u) => ({ ...u, rollback: v }))}
              label="我理解回滚也是真实写入：从 snapshot 还原文件内容"
              desc="不会回退第三方应用对配置文件的额外修改"
            />
            <CheckRow
              checked={understood.irreversible_after_edit}
              onChange={(v) =>
                setUnderstood((u) => ({ ...u, irreversible_after_edit: v }))
              }
              label="我理解：若我手动改动了 Agent 配置后再回滚，可能丢失这些手动改动"
              desc="snapshot 只记录 ClawHeart 写入时刻的状态"
            />
          </div>

          <div className="mt-2 px-3 py-2.5 rounded-md bg-amber-500/10 border border-amber-500/30 text-[11.5px] text-amber-700 dark:text-amber-300 leading-relaxed">
            <strong>建议：</strong>
            首次开启前先查看 dry-run 沙箱目录的内容，确认 ClawHeart 写入的格式与你期望一致：
            <br />
            <code className="font-mono text-[11px]">
              open ~/.clawheart-v2/dry-run/
            </code>
          </div>
        </div>

        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center justify-end gap-2">
          <button
            onClick={onClose}
            disabled={busy}
            className="px-3.5 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            disabled={!allChecked || busy}
            className={cn(
              "flex items-center gap-1.5 px-4 py-1.5 rounded-md text-[12.5px] font-medium",
              allChecked && !busy
                ? "bg-amber-500 text-white hover:bg-amber-600"
                : "bg-bg-elev2 text-text-muted cursor-not-allowed",
            )}
          >
            {busy && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
            启用实际写入
          </button>
        </footer>
      </div>
    </div>
  );
}

function CheckRow({
  checked,
  onChange,
  label,
  desc,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  desc?: string;
}) {
  return (
    <label
      className={cn(
        "flex items-start gap-2 px-3 py-2 rounded-md border cursor-pointer transition-colors",
        checked
          ? "border-emerald-500/30 bg-emerald-500/5"
          : "border-border-soft hover:border-text-muted",
      )}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="mt-0.5 w-3.5 h-3.5 accent-emerald-500 flex-shrink-0"
      />
      <div className="flex-1 min-w-0">
        <div className="text-[12.5px] text-text leading-snug">{label}</div>
        {desc && (
          <div className="text-[11px] text-text-muted mt-0.5 leading-snug">
            {desc}
          </div>
        )}
      </div>
    </label>
  );
}
