import { useState } from "react";
import { ShieldCheck, Loader2 } from "lucide-react";
import { toast } from "sonner";
import {
  useAccessMode,
  useCaStatus,
  useInstallCa,
  useSetAccessMode,
  useUninstallCa,
  type AccessTier,
  type CaInstallResult,
} from "@/hooks/useAccessMode";
import { TIERS, getTier } from "./data";
import { AccessTierCard } from "./AccessTierCard";
import { SwitchTierDialog } from "./SwitchTierDialog";
import { InstallCaDialog } from "./InstallCaDialog";
import { SandboxCommandDialog } from "./SandboxCommandDialog";
import { AgentTierOverrideList } from "./AgentTierOverrideList";

export function AccessModeTool() {
  const { data: mode, isLoading } = useAccessMode();
  const { data: caStatus } = useCaStatus();
  const setMode = useSetAccessMode();
  const installCa = useInstallCa();
  const uninstallCa = useUninstallCa();

  const [pendingTier, setPendingTier] = useState<AccessTier | null>(null);
  const [caInstallResult, setCaInstallResult] = useState<CaInstallResult | null>(
    null,
  );
  const [sandboxOpen, setSandboxOpen] = useState(false);

  const currentTier = mode?.current_tier ?? "tier1";
  const currentMeta = getTier(currentTier);

  if (isLoading || !mode) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        <Loader2 className="w-4 h-4 animate-spin mr-2" />
        加载中…
      </div>
    );
  }

  async function handleSelect(tier: AccessTier) {
    if (tier === currentTier) return;
    setPendingTier(tier);
  }

  async function handleConfirmSwitch() {
    if (!pendingTier) return;
    try {
      await setMode.mutateAsync(pendingTier);
      toast.success(`已切换为「${getTier(pendingTier).name}」`);
      setPendingTier(null);
    } catch (e) {
      toast.error(`切换失败：${e}`);
    }
  }

  async function handleInstallCa() {
    try {
      const result = await installCa.mutateAsync();
      setCaInstallResult(result);
    } catch (e) {
      toast.error(`安装失败：${e}`);
    }
  }

  async function handleUninstallCa() {
    if (
      !confirm(
        "确认卸载 CA 证书？卸载后「系统代理」模式无法解密 HTTPS 流量。",
      )
    )
      return;
    await uninstallCa.mutateAsync();
    toast.success("CA 已卸载");
  }

  const caBusy = installCa.isPending || uninstallCa.isPending;

  return (
    <div className="px-6 py-6">
      {/* Header banner: current tier + engine status */}
      <div className="mx-auto mb-6 max-w-[1100px]">
        <div className="flex items-center gap-3 px-4 py-3 rounded-xl bg-bg-elev border border-border">
          <ShieldCheck
            className="w-5 h-5 flex-shrink-0"
            style={{ color: `rgb(var(--tool-${currentMeta.color}))` }}
          />
          <div className="flex-1 min-w-0">
            <div className="text-[12px] text-text-muted uppercase tracking-wider mb-0.5">
              当前监控模式
            </div>
            <div className="text-[14px] font-semibold tracking-tight">
              {currentMeta.name}
              <span className="ml-2 text-[12px] font-normal text-text-dim">
                · {currentMeta.subtitle}
              </span>
            </div>
          </div>
          {mode.backend_ready ? (
            <span className="text-[11px] px-2 py-1 rounded-md bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
              ● 代理引擎已就绪
            </span>
          ) : currentTier === "tier3" ? (
            <span className="text-[11px] px-2 py-1 rounded-md bg-bg-elev2 text-text-muted border border-border">
              按需启动 · 见下方命令生成器
            </span>
          ) : currentTier === "tier2" && !mode.ca_installed ? (
            <span className="text-[11px] px-2 py-1 rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
              ⚠ 需先安装 CA 证书（见下方配置面板）
            </span>
          ) : (
            <span className="text-[11px] px-2 py-1 rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
              代理引擎启动中…（如长时间停留请重启应用或查看日志）
            </span>
          )}
        </div>
      </div>

      {/* 3-column tier comparison · 每档卡片内自带配置面板 */}
      <div className="mx-auto max-w-[1100px] grid grid-cols-1 md:grid-cols-3 gap-4">
        {TIERS.map((tier) => (
          <AccessTierCard
            key={tier.id}
            tier={tier}
            isCurrent={tier.id === currentTier}
            mode={mode}
            caStatus={caStatus}
            caBusy={caBusy}
            onSelect={() => handleSelect(tier.id)}
            onInstallCa={handleInstallCa}
            onUninstallCa={handleUninstallCa}
            onOpenSandbox={() => setSandboxOpen(true)}
          />
        ))}
      </div>

      {/* Agent 维度覆盖情况 */}
      <AgentTierOverrideList currentTier={currentTier} />

      {/* Dialogs */}
      {pendingTier && (
        <SwitchTierDialog
          fromTier={currentTier}
          toTier={pendingTier}
          fetchUrl={mode.fetch_url_template}
          caInstalled={mode.ca_installed}
          onConfirm={handleConfirmSwitch}
          onCancel={() => setPendingTier(null)}
          onInstallCa={handleInstallCa}
          onOpenSandboxHelper={() => setSandboxOpen(true)}
        />
      )}

      {caInstallResult && (
        <InstallCaDialog
          result={caInstallResult}
          caPath={mode.ca_path}
          onClose={() => setCaInstallResult(null)}
        />
      )}

      {sandboxOpen && (
        <SandboxCommandDialog onClose={() => setSandboxOpen(false)} />
      )}
    </div>
  );
}
