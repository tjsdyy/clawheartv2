import { Diamond, Pause, ExternalLink, LogOut, type LucideIcon } from "lucide-react";
import { useOverlays } from "@/hooks/useOverlays";
import { useStatus, useRecentEvents } from "@/hooks/useStatus";

export function TrayPopup() {
  const { trayOpen, closeTray } = useOverlays();
  const { data: status } = useStatus();
  const { data: events = [] } = useRecentEvents();

  if (!trayOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/40 backdrop-blur-sm z-[500] animate-fadein"
      onClick={closeTray}
    >
      <div
        className="absolute bottom-12 left-4 surface w-[320px] overflow-hidden"
        style={{ boxShadow: "0 16px 48px rgb(0 0 0 / 0.4)" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex justify-between items-center px-4 py-3.5 border-b border-border">
          <div className="flex items-center gap-2 font-semibold text-sm">
            <Diamond className="w-4 h-4 text-accent" style={{ filter: "drop-shadow(0 0 4px rgb(var(--accent) / 0.4))" }} />
            <span>ClawHeart</span>
          </div>
          <div className="flex items-center gap-1.5 text-[12px] text-text-dim">
            <span
              className="w-2 h-2 rounded-full"
              style={{
                background: status?.protected ? "rgb(var(--accent))" : "rgb(var(--critical))",
                boxShadow: status?.protected ? "0 0 6px rgb(var(--accent) / 0.5)" : "none",
              }}
            />
            <span>{status?.protected ? "防护中" : "已暂停"}</span>
          </div>
        </div>

        {/* Stats */}
        <div className="px-4 py-3 text-[12px] text-text-dim font-mono leading-[1.9] border-b border-border">
          Agent <strong className="text-text font-medium">{status?.agents ?? 0}</strong> ·{" "}
          MCP <strong className="text-text font-medium">{status?.mcp_servers ?? 0}</strong>
          <br />
          今日 <strong className="text-text font-medium">{((status?.today_requests ?? 0) / 1000).toFixed(1)}K</strong> req ·{" "}
          <strong className="text-text font-medium">${status?.today_cost_usd?.toFixed(2) ?? "0.00"}</strong>
        </div>

        {/* Events */}
        <div className="px-4 py-3 border-b border-border">
          <h5 className="text-[10px] uppercase tracking-wider text-text-muted mb-2.5 font-bold font-mono">
            最近事件
          </h5>
          {events.map((e) => (
            <div key={e.id} className="flex items-center gap-2 py-1.5 text-[12px]">
              <span
                className="w-[7px] h-[7px] rounded-full flex-shrink-0"
                style={{ background: `rgb(var(--${e.severity}))` }}
              />
              <span className="font-mono text-text-muted text-[11px] w-[38px]">{e.timestamp}</span>
              <span className="text-text-dim">{e.label}</span>
            </div>
          ))}
        </div>

        {/* Actions */}
        <div className="p-2">
          <TrayAction icon={Pause} label="暂停防护 5 分钟" onClick={closeTray} />
          <TrayAction icon={ExternalLink} label="打开主面板" onClick={closeTray} />
          <TrayAction icon={LogOut} label="退出" onClick={closeTray} />
        </div>
      </div>
    </div>
  );
}

function TrayAction({ icon: Icon, label, onClick }: { icon: LucideIcon; label: string; onClick?: () => void }) {
  return (
    <button
      onClick={onClick}
      className="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-[13px] text-text-dim hover:bg-bg-elev2 hover:text-text transition-colors"
    >
      <Icon className="w-[15px] h-[15px]" />
      <span>{label}</span>
    </button>
  );
}
