import { Check, X, ArrowRight, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";
import { AccessTierAsciiFlow } from "./AccessTierAsciiFlow";
import { TierConfigPanel } from "./TierConfigPanel";
import type { TierMeta } from "./data";
import type {
  AccessModeInfo,
  CaStatus,
} from "@/hooks/useAccessMode";

interface Props {
  tier: TierMeta;
  isCurrent: boolean;
  mode: AccessModeInfo;
  caStatus?: CaStatus;
  caBusy?: boolean;
  onSelect: () => void;
  onInstallCa: () => void;
  onUninstallCa: () => void;
  onOpenSandbox: () => void;
}

export function AccessTierCard({
  tier,
  isCurrent,
  mode,
  caStatus,
  caBusy,
  onSelect,
  onInstallCa,
  onUninstallCa,
  onOpenSandbox,
}: Props) {
  const Icon = tier.icon;
  return (
    <div
      className={cn(
        "flex flex-col rounded-xl border bg-bg-elev p-5 transition-colors",
        isCurrent
          ? "border-accent ring-1 ring-accent/30"
          : "border-border hover:border-text-muted",
      )}
    >
      {/* Header — icon + 名称 + 切换按钮 */}
      <div className="flex items-start gap-3 mb-3">
        <div
          className="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0"
          style={{ background: `rgb(var(--tool-${tier.color}) / 0.15)` }}
        >
          <Icon
            className="w-5 h-5"
            style={{ color: `rgb(var(--tool-${tier.color}))` }}
          />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 mb-0.5">
            <h3 className="text-[15px] font-semibold tracking-tight">
              {tier.name}
            </h3>
            {tier.recommended && (
              <span className="inline-flex items-center gap-0.5 text-[10px] font-medium px-1.5 py-0.5 rounded text-accent bg-accent/10">
                <Sparkles className="w-2.5 h-2.5" />
                推荐
              </span>
            )}
          </div>
          <div className="text-[11.5px] text-text-muted leading-snug">
            {tier.subtitle}
          </div>
        </div>
        <div className="flex-shrink-0">
          {isCurrent ? (
            <div className="px-2.5 py-1.5 text-[11.5px] font-medium text-accent bg-accent/10 rounded-md border border-accent/30 whitespace-nowrap">
              ✓ 当前
            </div>
          ) : (
            <button
              onClick={onSelect}
              className="flex items-center gap-1 px-2.5 py-1.5 text-[11.5px] font-medium text-text bg-bg-elev2 hover:bg-bg-elev2/70 rounded-md border border-border transition-colors whitespace-nowrap"
            >
              切换
              <ArrowRight className="w-3 h-3" />
            </button>
          )}
        </div>
      </div>

      {/* Description */}
      <p className="text-[12.5px] text-text-dim leading-relaxed mb-3">
        {tier.description}
      </p>

      {/* 适用场景 — 挪到图解上方，方便快速判断 */}
      <div className="mb-3 pb-3 border-b border-border-soft">
        <div className="text-[10.5px] text-text-muted uppercase tracking-wider mb-1">
          适用场景
        </div>
        <div className="text-[12px] text-text-dim leading-relaxed">{tier.bestFor}</div>
      </div>

      {/* 配置面板 — 挪到图解上方，便于直接配置 */}
      <div className="mb-3.5">
        <TierConfigPanel
          tier={tier.id}
          isCurrent={isCurrent}
          mode={mode}
          caStatus={caStatus}
          caBusy={caBusy}
          onInstallCa={onInstallCa}
          onUninstallCa={onUninstallCa}
          onOpenSandbox={onOpenSandbox}
        />
      </div>

      {/* ASCII Flow — 图解作为视觉理解辅助 */}
      <AccessTierAsciiFlow tier={tier.id} className="mb-3.5" />

      {/* Pros */}
      <div className="mb-2.5">
        <div className="text-[10.5px] text-text-muted uppercase tracking-wider mb-1.5">
          能力
        </div>
        <ul className="space-y-1">
          {tier.pros.map((p) => (
            <li
              key={p}
              className="flex items-start gap-1.5 text-[11.5px] text-text-dim leading-snug"
            >
              <Check className="w-3 h-3 mt-0.5 flex-shrink-0 text-emerald-500" />
              <span>{p}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Cons */}
      <div>
        <div className="text-[10.5px] text-text-muted uppercase tracking-wider mb-1.5">
          限制
        </div>
        <ul className="space-y-1">
          {tier.cons.map((c) => (
            <li
              key={c}
              className="flex items-start gap-1.5 text-[11.5px] text-text-muted leading-snug"
            >
              <X className="w-3 h-3 mt-0.5 flex-shrink-0 text-amber-500" />
              <span>{c}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
