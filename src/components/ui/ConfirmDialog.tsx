/**
 * 内嵌确认对话框 —— 替代 window.confirm（Tauri webview 不支持）
 *
 * 用法：
 *   const [confirmOpen, setConfirmOpen] = useState<{ ... } | null>(null);
 *   setConfirmOpen({ title, message, dangerous: true, onConfirm: () => {...} });
 *   {confirmOpen && <ConfirmDialog {...confirmOpen} onCancel={() => setConfirmOpen(null)} />}
 */
import { AlertTriangle, X, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

export interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  dangerous?: boolean;
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  title,
  message,
  confirmText = "确认",
  cancelText = "取消",
  dangerous = false,
  loading = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  return (
    <div
      className="fixed inset-0 z-[60] bg-black/40 flex items-center justify-center p-6 animate-fadein"
      onClick={onCancel}
    >
      <div
        className="bg-bg rounded-xl shadow-2xl border border-border max-w-md w-full overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <h3 className="text-[14px] font-semibold tracking-tight flex items-center gap-2">
            {dangerous && (
              <AlertTriangle className="w-4 h-4 text-critical" />
            )}
            {title}
          </h3>
          <button
            onClick={onCancel}
            className="text-text-muted hover:text-text"
          >
            <X className="w-4 h-4" />
          </button>
        </header>
        <div className="px-5 py-5">
          <p className="text-[12.5px] text-text-dim leading-relaxed whitespace-pre-line">
            {message}
          </p>
        </div>
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center justify-end gap-2">
          <button
            onClick={onCancel}
            disabled={loading}
            className="px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2 disabled:opacity-50"
          >
            {cancelText}
          </button>
          <button
            onClick={onConfirm}
            disabled={loading}
            className={cn(
              "flex items-center gap-1.5 px-4 py-1.5 rounded-md text-[12.5px] font-medium text-white disabled:opacity-50",
              dangerous
                ? "bg-critical hover:bg-critical/90"
                : "bg-accent hover:bg-accent/90",
            )}
          >
            {loading && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
            {confirmText}
          </button>
        </footer>
      </div>
    </div>
  );
}
