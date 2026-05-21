import { cn } from "@/lib/utils";
import type { AccessTier } from "@/hooks/useAccessMode";

interface Props {
  tier: AccessTier;
  className?: string;
}

const TIER_COLOR: Record<AccessTier, string> = {
  tier1: "monitor",
  tier2: "scan",
  tier3: "advisory",
};

const NODE_FILL = "rgb(var(--bg-elev2))";
const NODE_STROKE = "rgb(var(--border))";
const TEXT = "rgb(var(--text))";
const MUTED = "rgb(var(--text-muted))";

export function AccessTierAsciiFlow({ tier, className }: Props) {
  const color = `rgb(var(--tool-${TIER_COLOR[tier]}))`;
  const colorBg = `rgb(var(--tool-${TIER_COLOR[tier]}) / 0.10)`;
  const markerId = `flow-arrow-${tier}`;

  return (
    <div
      className={cn(
        "bg-bg/40 border border-border-soft rounded-md px-3 py-3",
        className,
      )}
    >
      <svg
        viewBox="0 0 320 110"
        className="block w-full h-[110px]"
        preserveAspectRatio="xMidYMid meet"
      >
        <defs>
          <marker
            id={markerId}
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="5"
            markerHeight="5"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 5 L 0 10 z" fill={MUTED} />
          </marker>
        </defs>
        {tier === "tier1" && (
          <Tier1Flow color={color} colorBg={colorBg} markerId={markerId} />
        )}
        {tier === "tier2" && (
          <Tier2Flow color={color} colorBg={colorBg} markerId={markerId} />
        )}
        {tier === "tier3" && (
          <Tier3Flow color={color} colorBg={colorBg} markerId={markerId} />
        )}
      </svg>
    </div>
  );
}

interface FlowProps {
  color: string;
  colorBg: string;
  markerId: string;
}

// ──────────────────────────────────────────────────────────────────
// Tier 1: 单一工具显式接入 → 反向代理 → 互联网
// ──────────────────────────────────────────────────────────────────
function Tier1Flow({ color, colorBg, markerId }: FlowProps) {
  return (
    <g>
      <NormalNode x={14} y={40} w={58} h={30} label="工具" sub="改 URL" />
      <Arrow d="M 72 55 L 108 55" markerId={markerId} />
      <ClawNode
        x={110}
        y={28}
        w={100}
        h={54}
        title="反向代理"
        sub=":19112 · HTTP"
        color={color}
        colorBg={colorBg}
      />
      <Arrow d="M 210 55 L 246 55" markerId={markerId} />
      <CloudNode x={248} y={40} w={58} h={30} />
    </g>
  );
}

// ──────────────────────────────────────────────────────────────────
// Tier 2: 多应用汇聚 → MITM → 互联网
// ──────────────────────────────────────────────────────────────────
function Tier2Flow({ color, colorBg, markerId }: FlowProps) {
  return (
    <g>
      <NormalNode x={14} y={6} w={58} h={20} label="应用 A" small />
      <NormalNode x={14} y={45} w={58} h={20} label="应用 B" small />
      <NormalNode x={14} y={84} w={58} h={20} label="应用 C" small />
      {/* 汇聚到 ClawHeart 左侧 (110, 55) */}
      <Arrow d="M 72 16 Q 92 16 108 42" markerId={markerId} />
      <Arrow d="M 72 55 L 108 55" markerId={markerId} />
      <Arrow d="M 72 94 Q 92 94 108 68" markerId={markerId} />
      <ClawNode
        x={110}
        y={28}
        w={100}
        h={54}
        title="MITM 代理"
        sub=":19111 · TLS+CA"
        color={color}
        colorBg={colorBg}
      />
      <Arrow d="M 210 55 L 246 55" markerId={markerId} />
      <CloudNode x={248} y={40} w={58} h={30} />
    </g>
  );
}

// ──────────────────────────────────────────────────────────────────
// Tier 3: 沙箱包围 agent，唯一出口 → 强制代理 → 互联网
// ──────────────────────────────────────────────────────────────────
function Tier3Flow({ color, colorBg, markerId }: FlowProps) {
  return (
    <g>
      {/* 沙箱虚线框 */}
      <rect
        x={10}
        y={12}
        width={92}
        height={86}
        rx={6}
        fill="rgb(var(--bg-elev) / 0.6)"
        stroke={MUTED}
        strokeWidth={1}
        strokeDasharray="4 3"
      />
      {/* 锁图标 + 沙箱标题 */}
      <g transform="translate(22, 18)">
        <rect
          x={0}
          y={3}
          width={6}
          height={5}
          rx={0.8}
          fill="none"
          stroke={MUTED}
          strokeWidth={0.9}
        />
        <path
          d="M 1.2 3 V 1.6 a 1.8 1.8 0 0 1 3.6 0 V 3"
          fill="none"
          stroke={MUTED}
          strokeWidth={0.9}
        />
        <text x={10} y={8} fontSize={10} fill={MUTED} fontWeight={500}>
          沙箱
        </text>
      </g>
      {/* agent 节点 */}
      <NormalNode x={22} y={42} w={68} h={26} label="agent" />
      {/* 唯一出口标签 */}
      <text
        x={56}
        y={86}
        textAnchor="middle"
        fontSize={8.5}
        fill={MUTED}
      >
        唯一出口 ↓
      </text>
      {/* 出口箭头 → ClawHeart */}
      <Arrow d="M 102 55 L 108 55" markerId={markerId} />
      <ClawNode
        x={110}
        y={28}
        w={100}
        h={54}
        title="强制出口"
        sub=":19113 · 隔离"
        color={color}
        colorBg={colorBg}
      />
      <Arrow d="M 210 55 L 246 55" markerId={markerId} />
      <CloudNode x={248} y={40} w={58} h={30} />
    </g>
  );
}

// ──────────────────────────────────────────────────────────────────
// 节点组件
// ──────────────────────────────────────────────────────────────────
function NormalNode({
  x,
  y,
  w,
  h,
  label,
  sub,
  small,
}: {
  x: number;
  y: number;
  w: number;
  h: number;
  label: string;
  sub?: string;
  small?: boolean;
}) {
  const cx = x + w / 2;
  const cy = y + h / 2;
  return (
    <g>
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={4}
        fill={NODE_FILL}
        stroke={NODE_STROKE}
        strokeWidth={1}
      />
      {sub ? (
        <>
          <text
            x={cx}
            y={cy - 3}
            textAnchor="middle"
            dominantBaseline="middle"
            fontSize={10}
            fill={TEXT}
          >
            {label}
          </text>
          <text
            x={cx}
            y={cy + 8}
            textAnchor="middle"
            dominantBaseline="middle"
            fontSize={8}
            fill={MUTED}
          >
            {sub}
          </text>
        </>
      ) : (
        <text
          x={cx}
          y={cy}
          textAnchor="middle"
          dominantBaseline="middle"
          fontSize={small ? 9 : 10}
          fill={TEXT}
        >
          {label}
        </text>
      )}
    </g>
  );
}

function ClawNode({
  x,
  y,
  w,
  h,
  title,
  sub,
  color,
  colorBg,
}: {
  x: number;
  y: number;
  w: number;
  h: number;
  title: string;
  sub: string;
  color: string;
  colorBg: string;
}) {
  const cx = x + w / 2;
  const cy = y + h / 2;
  return (
    <g>
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={6}
        fill={colorBg}
        stroke={color}
        strokeWidth={1.3}
      />
      {/* 左上角小色标 */}
      <rect x={x + 6} y={y + 6} width={6} height={6} rx={1.5} fill={color} />
      <text
        x={cx}
        y={cy - 4}
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize={11}
        fontWeight={600}
        fill={color}
      >
        {title}
      </text>
      <text
        x={cx}
        y={cy + 10}
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize={8.5}
        fill={MUTED}
      >
        {sub}
      </text>
    </g>
  );
}

function CloudNode({
  x,
  y,
  w,
  h,
}: {
  x: number;
  y: number;
  w: number;
  h: number;
}) {
  return (
    <g>
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={4}
        fill="transparent"
        stroke={NODE_STROKE}
        strokeWidth={1}
        strokeDasharray="3 2"
      />
      <text
        x={x + w / 2}
        y={y + h / 2}
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize={10}
        fill={MUTED}
      >
        互联网
      </text>
    </g>
  );
}

function Arrow({ d, markerId }: { d: string; markerId: string }) {
  return (
    <path
      d={d}
      stroke={MUTED}
      strokeWidth={1}
      fill="none"
      markerEnd={`url(#${markerId})`}
    />
  );
}
