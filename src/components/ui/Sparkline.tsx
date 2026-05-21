/**
 * 极简 SVG sparkline — 无外部依赖。
 * Usage: <Sparkline data={[1,2,3,2,5]} color="rgb(var(--accent))" />
 */
interface Props {
  data: number[];
  width?: number;
  height?: number;
  color?: string;
  fill?: boolean;
  strokeWidth?: number;
}

export function Sparkline({
  data,
  width = 120,
  height = 36,
  color = "rgb(var(--accent))",
  fill = true,
  strokeWidth = 1.5,
}: Props) {
  if (data.length === 0) {
    return (
      <svg width={width} height={height} className="opacity-30">
        <line
          x1={0}
          y1={height / 2}
          x2={width}
          y2={height / 2}
          stroke="rgb(var(--text-muted))"
          strokeWidth="1"
          strokeDasharray="2 3"
        />
      </svg>
    );
  }

  const max = Math.max(...data, 1);
  const min = Math.min(...data, 0);
  const span = max - min || 1;
  const step = data.length > 1 ? width / (data.length - 1) : 0;

  // 留 2px padding 防止顶到边
  const pad = 2;
  const innerH = height - pad * 2;

  const points = data.map((v, i) => {
    const x = i * step;
    const y = pad + innerH - ((v - min) / span) * innerH;
    return [x, y] as [number, number];
  });

  const line = points.map(([x, y], i) => (i === 0 ? `M${x},${y}` : `L${x},${y}`)).join(" ");

  const area = fill
    ? `${line} L${points[points.length - 1][0]},${height} L0,${height} Z`
    : "";

  return (
    <svg width={width} height={height} style={{ display: "block" }}>
      {fill && area && <path d={area} fill={color} fillOpacity={0.12} />}
      <path d={line} fill="none" stroke={color} strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round" />
      {/* last point dot */}
      {points.length > 0 && (
        <circle cx={points[points.length - 1][0]} cy={points[points.length - 1][1]} r={2} fill={color} />
      )}
    </svg>
  );
}
