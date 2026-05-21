import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Radar, ArrowRight, Check } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useOnboarding } from "@/hooks/useOnboarding";
import {
  useSetAccessMode,
  type AccessTier,
} from "@/hooks/useAccessMode";
import { TIERS } from "@/components/access-mode/data";
import { AccessTierAsciiFlow } from "@/components/access-mode/AccessTierAsciiFlow";
import type { DiscoveredAgent } from "@/hooks/useAgents";
import { cn } from "@/lib/utils";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

type Step = "welcome" | "tier";

export function Onboarding() {
  const { t } = useTranslation();
  const setCompleted = useOnboarding((s) => s.setCompleted);
  const setMode = useSetAccessMode();
  const [step, setStep] = useState<Step>("welcome");
  const [discovering, setDiscovering] = useState(false);
  const [selectedTier, setSelectedTier] = useState<AccessTier>("tier1");
  const [committing, setCommitting] = useState(false);

  const handleDiscover = async () => {
    setDiscovering(true);
    try {
      if (inTauri) {
        const agents = await invoke<DiscoveredAgent[]>("discover_agents_now");
        toast.success(
          t("onboarding.found", {
            defaultValue: `发现 ${agents.length} 个 AI Agent`,
          }),
        );
      } else {
        await new Promise((r) => setTimeout(r, 600));
      }
      setStep("tier");
    } catch (e) {
      toast.error(`${e}`);
    } finally {
      setDiscovering(false);
    }
  };

  const handleFinish = async () => {
    setCommitting(true);
    try {
      await setMode.mutateAsync(selectedTier);
    } catch {
      // 容错：保存失败也允许进入应用
    } finally {
      setCommitting(false);
      setCompleted(true);
    }
  };

  if (step === "welcome") {
    return (
      <div className="flex items-center justify-center min-h-screen bg-bg px-8">
        <div className="text-center max-w-md animate-fadein">
          <div className="text-[56px] mb-6 leading-none">👋</div>
          <h1 className="text-[28px] font-semibold tracking-tight mb-3">
            {t("onboarding.welcome")}
          </h1>
          <p className="text-text-dim text-[14px] leading-relaxed mb-8">
            {t("onboarding.description")}
            <br />
            {t("onboarding.details")}
          </p>

          <button
            onClick={handleDiscover}
            disabled={discovering}
            className="btn-primary"
          >
            {discovering ? (
              <>
                <Radar className="w-4 h-4 animate-spin" />
                {t("onboarding.scanning")}
              </>
            ) : (
              <>
                <Radar className="w-4 h-4" />
                {t("onboarding.start_discover")}
                <ArrowRight className="w-3.5 h-3.5 ml-1" />
              </>
            )}
          </button>

          <button
            onClick={() => setStep("tier")}
            className="block mx-auto mt-4 text-text-muted text-[13px] px-4 py-2 hover:text-text-dim transition-colors"
          >
            {t("onboarding.skip")}
          </button>
        </div>
      </div>
    );
  }

  // step === "tier"
  return (
    <div className="flex items-center justify-center min-h-screen bg-bg px-6 py-10">
      <div className="w-full max-w-4xl animate-fadein">
        <div className="text-center mb-8">
          <div className="text-[40px] mb-3 leading-none">🎚️</div>
          <h1 className="text-[24px] font-semibold tracking-tight mb-2">
            选择监控模式
          </h1>
          <p className="text-text-dim text-[13.5px] leading-relaxed max-w-md mx-auto">
            选择 ClawHeart 对 AI 工具出站流量的覆盖策略。
            <br />
            可随时在「监控模式」工具页中调整。
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          {TIERS.map((tier) => {
            const Icon = tier.icon;
            const selected = selectedTier === tier.id;
            return (
              <button
                key={tier.id}
                onClick={() => setSelectedTier(tier.id)}
                className={cn(
                  "text-left rounded-xl border p-4 transition-colors flex flex-col",
                  selected
                    ? "border-accent ring-1 ring-accent/30 bg-accent/5"
                    : "border-border bg-bg-elev hover:border-text-muted",
                )}
              >
                <div className="flex items-start gap-2.5 mb-2">
                  <div
                    className="w-9 h-9 rounded-lg flex items-center justify-center flex-shrink-0"
                    style={{
                      background: `rgb(var(--tool-${tier.color}) / 0.15)`,
                    }}
                  >
                    <Icon
                      className="w-4.5 h-4.5"
                      style={{ color: `rgb(var(--tool-${tier.color}))` }}
                    />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <div className="text-[14px] font-semibold tracking-tight">
                        {tier.name}
                      </div>
                      {tier.recommended && (
                        <span className="text-[10px] font-medium px-1.5 py-0.5 rounded text-accent bg-accent/10">
                          推荐
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-text-muted mt-0.5">
                      {tier.subtitle}
                    </div>
                  </div>
                  {selected && (
                    <Check className="w-4 h-4 text-accent flex-shrink-0" />
                  )}
                </div>
                <p className="text-[12px] text-text-dim mb-2.5 leading-relaxed">
                  {tier.description}
                </p>
                <AccessTierAsciiFlow tier={tier.id} className="mb-2.5" />
                <ul className="space-y-0.5 text-[11px] text-text-dim leading-snug mt-auto">
                  {tier.pros.slice(0, 2).map((p) => (
                    <li key={p} className="flex items-start gap-1">
                      <span className="text-emerald-500">✓</span>
                      <span>{p}</span>
                    </li>
                  ))}
                </ul>
              </button>
            );
          })}
        </div>

        <div className="flex items-center justify-between gap-4">
          <button
            onClick={() => setCompleted(true)}
            className="text-text-muted text-[12.5px] px-3 py-2 hover:text-text-dim transition-colors"
          >
            稍后再选 · 默认「端点映射」
          </button>

          <button
            onClick={handleFinish}
            disabled={committing}
            className="btn-primary"
          >
            {committing ? (
              <>保存中…</>
            ) : (
              <>
                进入 ClawHeart
                <ArrowRight className="w-3.5 h-3.5 ml-1" />
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
