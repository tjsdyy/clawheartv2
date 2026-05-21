import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import type { ToolDef } from "./tools.config";

/**
 * 工具卡片 — 扁平 + 彩色。
 *
 * - 每个工具有自己的色 token（--tool-{color}），通过 inline style 注入到 --c
 * - 默认底色 = bg-elev；hover 时 background = 6% tint + border 升为工具色
 * - icon 容器始终是工具色 12% tint + 工具色 stroke
 * - 用 JS 控制 hover 状态（避免 Tailwind arbitrary 不支持 color-mix）
 */
export function ToolCard({ tool }: { tool: ToolDef }) {
  const navigate = useNavigate();
  const [hovered, setHovered] = useState(false);
  const isSoon = tool.status === "coming_soon";
  const isAdd = tool.id === "more";
  const Icon = tool.icon;

  const colorVar = `var(--tool-${tool.color})`;
  const accent = `rgb(${colorVar})`;
  const tint = `color-mix(in srgb, ${accent} 6%, rgb(var(--bg-elev)))`;

  const cardStyle: React.CSSProperties = {
    background: !isSoon && hovered ? tint : "rgb(var(--bg-elev))",
    borderColor: !isSoon && hovered ? accent : "rgb(var(--border))",
    transform: !isSoon && hovered ? "translateY(-4px)" : "translateY(0)",
  };

  return (
    <button
      onClick={() => !isSoon && navigate(`/tools/${tool.id}`)}
      disabled={isSoon}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={cardStyle}
      className={cn(
        "tool-card group relative aspect-square text-left rounded-2xl p-3.5",
        "border transition-all duration-200",
        !isSoon && "cursor-pointer",
        isSoon && !isAdd && "opacity-50 cursor-not-allowed",
        isAdd && "border-dashed opacity-60",
      )}
    >
      {/* Icon */}
      <div
        className={cn(
          "w-10 h-10 rounded-xl flex items-center justify-center transition-transform",
          !isSoon && hovered && "scale-110",
        )}
        style={{
          background: `color-mix(in srgb, ${accent} 12%, transparent)`,
          color: accent,
        }}
      >
        <Icon className="w-5 h-5" strokeWidth={2.2} />
      </div>

      {/* Text */}
      <div className="mt-auto pt-2.5">
        <div className="font-semibold text-[13.5px] leading-tight tracking-tight text-text">
          {tool.label}
        </div>
        <div className="font-mono text-[10.5px] text-text-dim leading-snug mt-0.5">
          {tool.description}
        </div>
      </div>

      {/* Badge */}
      {tool.badge && <Badge kind={tool.badge.kind} value={tool.badge.value} />}
    </button>
  );
}

function Badge({ kind, value }: { kind: string; value?: string }) {
  if (kind === "soon") {
    return (
      <span className="chip absolute top-2.5 right-2.5 bg-bg-elev2 border border-border text-text-muted">
        {value}
      </span>
    );
  }
  if (kind === "alert" || kind === "alert-high") {
    const bg = kind === "alert" ? "rgb(var(--critical))" : "rgb(var(--high))";
    const shadow = kind === "alert" ? "rgb(var(--critical) / 0.3)" : "rgb(var(--high) / 0.3)";
    return (
      <span
        className="absolute top-2.5 right-2.5 pl-4 pr-2 py-[3px] rounded-full text-[9.5px] font-bold font-mono text-white"
        style={{ background: bg, boxShadow: `0 2px 8px ${shadow}` }}
      >
        <span
          className="absolute left-1.5 top-1/2 w-1.5 h-1.5 rounded-full bg-white"
          style={{ animation: "alertpulse 1.4s infinite" }}
        />
        {value}
      </span>
    );
  }
  if (kind === "count") {
    return (
      <span className="absolute top-3 right-3 text-[10px] font-semibold font-mono text-text-dim">
        {value}
      </span>
    );
  }
  return null;
}
