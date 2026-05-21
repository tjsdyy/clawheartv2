import { useState } from "react";
import { X, Lock, Copy, Loader2 } from "lucide-react";
import { toast } from "sonner";
import {
  generateSandboxCommand,
  type SandboxCommandPreview,
} from "@/hooks/useAccessMode";

interface Props {
  onClose: () => void;
}

export function SandboxCommandDialog({ onClose }: Props) {
  const [cmd, setCmd] = useState("python");
  const [args, setArgs] = useState("my_agent.py");
  const [preview, setPreview] = useState<SandboxCommandPreview | null>(null);
  const [loading, setLoading] = useState(false);

  async function regenerate() {
    setLoading(true);
    try {
      const argList = args.split(/\s+/).filter(Boolean);
      const result = await generateSandboxCommand(cmd, argList);
      setPreview(result);
    } finally {
      setLoading(false);
    }
  }

  function copyCommand() {
    if (!preview) return;
    navigator.clipboard.writeText(preview.command);
    toast.success("命令已复制");
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-lg bg-bg rounded-xl shadow-2xl border border-border overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            <Lock className="w-4 h-4 text-accent" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              沙箱命令生成器
            </h3>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="px-5 py-5 space-y-4">
          <div>
            <div className="text-[12px] text-text-muted uppercase tracking-wider mb-2">
              要包裹的命令
            </div>
            <div className="grid grid-cols-[120px_1fr] gap-2">
              <input
                type="text"
                value={cmd}
                onChange={(e) => setCmd(e.target.value)}
                placeholder="python"
                className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[12.5px] font-mono outline-none focus:border-accent"
              />
              <input
                type="text"
                value={args}
                onChange={(e) => setArgs(e.target.value)}
                placeholder="my_agent.py --verbose"
                className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[12.5px] font-mono outline-none focus:border-accent"
              />
            </div>
          </div>

          <button
            onClick={regenerate}
            disabled={loading}
            className="w-full py-2 rounded-md bg-bg-elev2 text-[12.5px] font-medium border border-border hover:border-text-muted disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {loading ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              "生成命令"
            )}
          </button>

          {preview && (
            <>
              <div>
                <div className="text-[12px] text-text-muted uppercase tracking-wider mb-2">
                  完整命令（已为 {preview.platform} 优化）
                </div>
                <div className="flex items-center gap-2 bg-bg-elev rounded-md border border-border px-3 py-2.5 font-mono text-[11.5px]">
                  <code className="flex-1 truncate text-text">
                    {preview.command}
                  </code>
                  <button
                    onClick={copyCommand}
                    className="flex items-center gap-1 text-text-muted hover:text-text text-[11px]"
                  >
                    <Copy className="w-3 h-3" />
                    复制
                  </button>
                </div>
              </div>

              {preview.notes.length > 0 && (
                <ul className="space-y-1 bg-bg-elev/40 rounded-md px-3 py-2.5 border border-border-soft">
                  {preview.notes.map((note, i) => (
                    <li
                      key={i}
                      className="text-[11.5px] text-text-dim leading-relaxed flex items-start gap-1.5"
                    >
                      <span className="text-text-muted">›</span>
                      <span>{note}</span>
                    </li>
                  ))}
                </ul>
              )}

              {!preview.feature_available && (
                <div className="bg-amber-500/10 border border-amber-500/30 rounded-md px-3.5 py-3 text-[11.5px] leading-relaxed text-amber-700 dark:text-amber-300">
                  <strong>预览模式：</strong>
                  沙箱隔离模式将于 W20 接入。当前命令可保存供后续直接执行。
                </div>
              )}
            </>
          )}
        </div>

        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex justify-end">
          <button
            onClick={onClose}
            className="px-3.5 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            关闭
          </button>
        </footer>
      </div>
    </div>
  );
}
