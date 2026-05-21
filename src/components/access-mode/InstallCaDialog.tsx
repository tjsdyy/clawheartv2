import { X, ShieldCheck, Copy } from "lucide-react";
import { toast } from "sonner";
import type { CaInstallResult } from "@/hooks/useAccessMode";

interface Props {
  result: CaInstallResult;
  caPath: string;
  onClose: () => void;
}

export function InstallCaDialog({ result, caPath, onClose }: Props) {
  const platformName =
    result.platform === "macos"
      ? "macOS"
      : result.platform === "windows"
        ? "Windows"
        : result.platform === "linux"
          ? "Linux"
          : "未知平台";

  function copyPath() {
    navigator.clipboard.writeText(caPath);
    toast.success("路径已复制");
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-lg bg-bg rounded-xl shadow-2xl border border-border overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            <ShieldCheck className="w-4 h-4 text-emerald-500" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              CA 证书 · {platformName} 安装步骤
            </h3>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="px-5 py-5 max-h-[60vh] overflow-y-auto space-y-4">
          <div className="bg-emerald-500/10 border border-emerald-500/30 rounded-md px-3.5 py-3 text-[12.5px] leading-relaxed">
            <div className="font-medium text-emerald-700 dark:text-emerald-300 mb-1">
              ✓ {result.message}
            </div>
            {result.fingerprint && (
              <div className="text-[11px] font-mono text-text-dim">
                指纹：{result.fingerprint}
              </div>
            )}
          </div>

          <div>
            <div className="text-[12px] text-text-muted uppercase tracking-wider mb-2">
              证书路径
            </div>
            <div className="flex items-center gap-2 bg-bg-elev rounded-md border border-border px-3 py-2 font-mono text-[11.5px]">
              <code className="flex-1 truncate text-text">{caPath}</code>
              <button
                onClick={copyPath}
                className="flex items-center gap-1 text-text-muted hover:text-text text-[11px]"
              >
                <Copy className="w-3 h-3" />
                复制
              </button>
            </div>
          </div>

          <div>
            <div className="text-[12px] text-text-muted uppercase tracking-wider mb-2">
              手动安装步骤（自动安装失败时使用）
            </div>
            <ol className="space-y-1.5">
              {result.manual_steps.map((step, i) => (
                <li
                  key={i}
                  className="flex items-start gap-2.5 text-[12px] text-text-dim leading-relaxed"
                >
                  <span className="w-5 h-5 rounded-full bg-bg-elev2 text-[10.5px] flex items-center justify-center font-medium text-text flex-shrink-0 mt-0.5">
                    {i + 1}
                  </span>
                  <span>{step}</span>
                </li>
              ))}
            </ol>
          </div>

          <div className="bg-amber-500/10 border border-amber-500/30 rounded-md px-3.5 py-3 text-[11.5px] leading-relaxed text-amber-700 dark:text-amber-300">
            <strong>信任声明：</strong>
            将自签 CA 加入受信任根证书后，ClawHeart 即可对 HTTPS 流量执行 TLS 终止。
            若需撤销信任：
            <br />
            <code className="font-mono text-[11px]">设置 → 代理 → 卸载 CA</code>
          </div>
        </div>

        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex justify-end">
          <button
            onClick={onClose}
            className="px-3.5 py-1.5 rounded-md text-[12.5px] font-medium bg-accent text-white hover:bg-accent/90"
          >
            完成
          </button>
        </footer>
      </div>
    </div>
  );
}
