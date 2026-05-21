// 工具矩阵的配置驱动列表。
// 5 张极简版（v2 alpha）—— OpenClaw 视为 Agent 一种，由 Agent 卡片统一管理。
// 路由本身保留，便于 ⌘K / 直接 URL 访问。
import {
  Activity,
  ScanLine,
  Wrench,
  Users,
  Settings,
  Wallet,
  KeyRound,
  type LucideIcon,
} from "lucide-react";

export type BadgeKind = "alert" | "alert-high" | "count" | "soon" | "none";

export interface ToolDef {
  id: string;
  label: string;
  description: string;
  icon: LucideIcon;
  /** Color token name from globals.css (--tool-*) */
  color: string;
  status: "active" | "coming_soon";
  badge?: { kind: BadgeKind; value?: string };
  /** Optional version target for soon items, shown as tag */
  version?: string;
}

export const TOOLS: ToolDef[] = [
  {
    id: "agents",
    label: "Agent 发现",
    description: "6 已发现",
    icon: Users,
    color: "logs",
    status: "active",
    badge: { kind: "count", value: "6" },
  },
  {
    id: "monitor",
    label: "实时监控",
    description: "实时流 · 拦截 · 用量",
    icon: Activity,
    color: "monitor",
    status: "active",
    badge: { kind: "alert", value: "2 新" },
  },
  {
    id: "scan",
    label: "安全扫描",
    description: "80 项安全审计",
    icon: ScanLine,
    color: "scan",
    status: "active",
    badge: { kind: "count", value: "2 ●" },
  },
  {
    id: "skills",
    label: "技能管理",
    description: "发现 · 鉴定 · 备份",
    icon: Wrench,
    color: "skills",
    status: "active",
  },
  {
    id: "providers",
    label: "渠道管理",
    description: "凭据库 · 分配",
    icon: KeyRound,
    color: "openclaw",
    status: "active",
  },
  {
    id: "usage",
    label: "用量统计",
    description: "Token · 成本 · 趋势",
    icon: Wallet,
    color: "budget",
    status: "active",
  },
  {
    id: "settings",
    label: "设置",
    description: "通用 · 安全 · 高级",
    icon: Settings,
    color: "audit",
    status: "active",
  },
];
