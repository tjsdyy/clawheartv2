import { useState } from "react";
import { X, AlertTriangle, Copy, ShieldCheck, Lock } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import type { AccessTier } from "@/hooks/useAccessMode";
import { getTier } from "./data";

interface Props {
  fromTier: AccessTier;
  toTier: AccessTier;
  fetchUrl: string;
  caInstalled: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  onInstallCa: () => void;
  onOpenSandboxHelper: () => void;
}

export function SwitchTierDialog({
  fromTier,
  toTier,
  fetchUrl,
  caInstalled,
  onConfirm,
  onCancel,
  onInstallCa,
  onOpenSandboxHelper,
}: Props) {
  const [step, setStep] = useState(0);
  const target = getTier(toTier);
  const isDowngrade =
    ["tier1", "tier2", "tier3"].indexOf(toTier) <
    ["tier1", "tier2", "tier3"].indexOf(fromTier);

  function copy(text: string, hint = "已复制") {
    navigator.clipboard.writeText(text);
    toast.success(hint);
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-xl bg-bg rounded-xl shadow-2xl border border-border overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            {toTier === "tier2" ? (
              <ShieldCheck className="w-4 h-4 text-accent" />
            ) : toTier === "tier3" ? (
              <Lock className="w-4 h-4 text-accent" />
            ) : (
              <AlertTriangle className="w-4 h-4 text-amber-500" />
            )}
            <h3 className="text-[14px] font-semibold tracking-tight">
              {isDowngrade ? "降级到" : "切换到"} {target.name}
            </h3>
          </div>
          <button
            onClick={onCancel}
            className="text-text-muted hover:text-text"
          >
            <X className="w-4 h-4" />
          </button>
        </header>

        <div className="px-5 py-5 max-h-[60vh] overflow-y-auto">
          {/* tier1: 改 base_url */}
          {toTier === "tier1" && (
            <div className="space-y-4">
              <Para>
                「端点映射」模式不要求安装证书或修改系统配置。将 AI 工具的 API Base URL
                指向以下本地反向代理端点即可：
              </Para>
              <CopyBlock value={fetchUrl} onCopy={() => copy(fetchUrl)} />
              <Para>
                常见工具配置方式：
              </Para>
              <ul className="space-y-1.5 text-[12.5px] text-text-dim">
                <Bullet>
                  <code className="font-mono text-[11.5px]">Cursor / Continue.dev</code>
                  ：设置 → 模型 → Base URL
                </Bullet>
                <Bullet>
                  <code className="font-mono text-[11.5px]">Python openai SDK</code>
                  ：<code className="font-mono text-[11.5px]">OpenAI(base_url="…")</code>
                </Bullet>
                <Bullet>
                  <code className="font-mono text-[11.5px]">环境变量</code>
                  ：<code className="font-mono text-[11.5px]">OPENAI_API_BASE</code>
                </Bullet>
              </ul>
              {isDowngrade && (
                <Notice>
                  降级后系统代理与 CA 证书<strong>不会自动卸载</strong>，可在「设置 →
                  代理」中手动清除。
                </Notice>
              )}
            </div>
          )}

          {/* tier2: 装 CA + 系统代理 */}
          {toTier === "tier2" && (
            <div className="space-y-4">
              <StepHeader
                index={1}
                title="安装 ClawHeart 根证书"
                done={caInstalled}
              />
              {!caInstalled ? (
                <div className="space-y-2">
                  <Para>
                    系统需将 ClawHeart 自签 CA 列入受信任根证书，方能对 HTTPS 流量执行
                    TLS 终止与内容审计。
                  </Para>
                  <button
                    onClick={onInstallCa}
                    className="px-3.5 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90"
                  >
                    自动安装 CA 证书 →
                  </button>
                </div>
              ) : (
                <Para className="text-emerald-500">
                  ✓ CA 已位于受信任根证书存储区，可继续下一步
                </Para>
              )}

              <StepHeader index={2} title="启用系统级 HTTPS 代理" done={step >= 2} />
              <Para>
                ClawHeart 将系统 HTTPS 代理设置为
                <code className="mx-1 font-mono text-[11.5px]">127.0.0.1:19111</code>
                ，遵循系统代理的应用程序将自动经由本地代理出网。
              </Para>

              <Notice variant="warn">
                <strong>限制说明：</strong>
                启用 TLS Pinning 的客户端（部分移动端 SDK、严格 Electron 应用）
                会拒绝接受自签 CA，此类流量无法被审计。
              </Notice>
            </div>
          )}

          {/* tier3: sandbox */}
          {toTier === "tier3" && (
            <div className="space-y-4">
              <Para>
                「沙箱隔离」模式通过
                <code className="mx-1 font-mono text-[11.5px]">clawheart sandbox</code>
                子命令启动目标进程，由 OS 沙箱机制强制约束其网络出口，实现进程级
                100% 流量覆盖。
              </Para>
              <CopyBlock
                value="clawheart sandbox -- python my_agent.py"
                onCopy={() =>
                  copy("clawheart sandbox -- python my_agent.py", "命令已复制")
                }
              />
              <Para>
                如需为自定义命令生成对应的 sandbox 包裹形式，可使用命令生成器：
              </Para>
              <button
                onClick={onOpenSandboxHelper}
                className="px-3.5 py-1.5 rounded-md bg-bg-elev2 text-[12.5px] font-medium border border-border hover:border-text-muted"
              >
                打开命令生成器 →
              </button>
              <Notice variant="warn">
                <strong>阶段说明：</strong>
                沙箱隔离能力将于 W20 接入；当前显示为预览态。Windows 平台
                （AppContainer + WFP）支持计划于 v2.1 发布。
              </Notice>
            </div>
          )}
        </div>

        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center justify-end gap-2">
          <button
            onClick={onCancel}
            className="px-3.5 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={() => {
              setStep(99);
              onConfirm();
            }}
            className={cn(
              "px-3.5 py-1.5 rounded-md text-[12.5px] font-medium",
              "bg-accent text-white hover:bg-accent/90",
            )}
          >
            确认切换到 {target.name}
          </button>
        </footer>
      </div>
    </div>
  );
}

function Para({ children, className }: { children: React.ReactNode; className?: string }) {
  return <p className={cn("text-[12.5px] text-text-dim leading-relaxed", className)}>{children}</p>;
}

function Bullet({ children }: { children: React.ReactNode }) {
  return (
    <li className="flex items-start gap-2">
      <span className="text-text-muted">›</span>
      <span>{children}</span>
    </li>
  );
}

function StepHeader({ index, title, done }: { index: number; title: string; done: boolean }) {
  return (
    <div className="flex items-center gap-2">
      <span
        className={cn(
          "w-5 h-5 rounded-full text-[10.5px] flex items-center justify-center font-medium",
          done ? "bg-emerald-500/15 text-emerald-500" : "bg-bg-elev2 text-text-dim",
        )}
      >
        {done ? "✓" : index}
      </span>
      <span className="text-[13px] font-medium">{title}</span>
    </div>
  );
}

function CopyBlock({ value, onCopy }: { value: string; onCopy: () => void }) {
  return (
    <div className="flex items-center gap-2 bg-bg-elev rounded-md border border-border px-3 py-2.5 font-mono text-[11.5px]">
      <code className="flex-1 truncate text-text">{value}</code>
      <button
        onClick={onCopy}
        className="flex items-center gap-1 text-text-muted hover:text-text text-[11px]"
      >
        <Copy className="w-3 h-3" />
        复制
      </button>
    </div>
  );
}

function Notice({
  children,
  variant = "info",
}: {
  children: React.ReactNode;
  variant?: "info" | "warn";
}) {
  return (
    <div
      className={cn(
        "text-[11.5px] leading-relaxed px-3 py-2.5 rounded-md border",
        variant === "warn"
          ? "bg-amber-500/10 border-amber-500/30 text-amber-700 dark:text-amber-300"
          : "bg-bg-elev border-border text-text-dim",
      )}
    >
      {children}
    </div>
  );
}
