import { useNavigate } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { useStatus } from "@/hooks/useStatus";
import { useAccessMode } from "@/hooks/useAccessMode";
import { getTier } from "@/components/access-mode/data";
import { HomeHero } from "./HomeHero";

export function ToolsGrid() {
  const navigate = useNavigate();
  const { data: status } = useStatus();
  const { data: accessMode } = useAccessMode();

  const currentTier = accessMode?.current_tier ?? "tier1";
  const currentTierMeta = getTier(currentTier);

  return (
    <div className="relative min-h-full flex flex-col items-center justify-center px-6 py-12">
      {/* 左上角：greeting + 统计 */}
      <div className="absolute top-3 left-6 z-10 flex items-center gap-2 text-[11.5px] font-mono text-text-muted">
        <span className="live-dot" />
        <strong className="font-medium text-text-dim font-sans">{greetingByHour()}</strong>
        <Sep />
        <span>今日 {status?.today_requests ?? 0} req</span>
        <Sep />
        <span>
          ${status?.today_cost_usd?.toFixed(2) ?? "0.00"}
          {status?.budget_limit_usd ? ` / $${status.budget_limit_usd.toFixed(2)}` : ""}
        </span>
        <Sep />
        <span>{status?.today_blocks ?? 0} 拦截</span>
      </div>

      {/* 右上角：监控模式 chip（紧靠主题图标左侧） */}
      <button
        onClick={() => navigate("/tools/access_mode")}
        className="absolute top-2 right-14 z-10 flex items-center gap-1.5 px-2.5 py-1 rounded-md font-sans text-[12px] text-text-dim hover:text-text bg-bg-elev/60 hover:bg-bg-elev2 border border-border transition-colors"
        title="切换监控模式"
      >
        <span
          className="w-1.5 h-1.5 rounded-full flex-shrink-0"
          style={{ background: `rgb(var(--tool-${currentTierMeta.color}))` }}
        />
        <span>
          监控模式 ·{" "}
          <strong className="font-medium text-text">{currentTierMeta.name}</strong>
        </span>
        <ChevronRight className="w-3 h-3 opacity-60" />
      </button>

      {/* 居中：Hero pipeline + 工具底栏 */}
      <div className="w-full" style={{ maxWidth: 1100 }}>
        <HomeHero />
      </div>
    </div>
  );
}

function greetingByHour(): string {
  const h = new Date().getHours();
  if (h < 6) return "凌晨好";
  if (h < 12) return "上午好";
  if (h < 14) return "中午好";
  if (h < 18) return "下午好";
  return "晚上好";
}

function Sep() {
  return <span className="opacity-40">·</span>;
}
