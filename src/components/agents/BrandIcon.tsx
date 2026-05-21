/**
 * 品牌图标 —— 主流厂商用 SVG path（来自 simple-icons），其他 fallback 到字母色块。
 *
 * 用法：<BrandIcon preset={preset} size={32} />
 * 或：<BrandIcon name="OpenAI" color="#10A37F" iconSvgPath="M22..." size={32} />
 */
import { cn } from "@/lib/utils";
import { BRAND_ICONS, presetInitial, type ProviderPreset } from "@/data/provider-presets";

interface Props {
  preset?: ProviderPreset;
  /** 当不传 preset 时，提供 name + color 模拟 */
  name?: string;
  color?: string;
  size?: number;
  /** 是否使用反色配色（用在选中态/品牌色背景上） */
  inverted?: boolean;
  className?: string;
  rounded?: "sm" | "md" | "lg";
}

export function BrandIcon({
  preset,
  name: nameProp,
  color: colorProp,
  size = 32,
  inverted = false,
  className,
  rounded = "md",
}: Props) {
  const color = preset?.color ?? colorProp ?? "#6B7280";
  const name = preset?.name ?? nameProp ?? "?";
  const path = preset ? BRAND_ICONS[preset.id] : undefined;
  const initial = preset ? presetInitial(preset) : name.charAt(0).toUpperCase();

  const radiusClass =
    rounded === "sm" ? "rounded-sm" : rounded === "lg" ? "rounded-lg" : "rounded-md";

  const bg = inverted ? color : `${color}1A`;
  const fg = inverted ? "#ffffff" : color;

  return (
    <div
      className={cn(
        "flex items-center justify-center flex-shrink-0",
        radiusClass,
        className,
      )}
      style={{
        width: size,
        height: size,
        background: bg,
        color: fg,
      }}
    >
      {path ? (
        <svg
          viewBox="0 0 24 24"
          width={Math.round(size * 0.6)}
          height={Math.round(size * 0.6)}
          fill="currentColor"
          aria-hidden="true"
        >
          <path d={path} />
        </svg>
      ) : (
        <span
          className="font-mono font-bold"
          style={{ fontSize: Math.max(10, Math.round(size * 0.42)) }}
        >
          {initial}
        </span>
      )}
    </div>
  );
}
