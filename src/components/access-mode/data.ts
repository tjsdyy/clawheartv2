import { Sparkles, ShieldCheck, Lock, type LucideIcon } from "lucide-react";
import type { AccessTier } from "@/hooks/useAccessMode";

export interface TierMeta {
  id: AccessTier;
  /** 监控模式名称（用户可见）*/
  name: string;
  /** 副标题：一句话技术定位 */
  subtitle: string;
  /** 模式工作机制描述（替代旧 metaphor）*/
  description: string;
  /** 图标 */
  icon: LucideIcon;
  /** --tool-* 颜色 token */
  color: string;
  /** 是否为推荐起始模式 */
  recommended?: boolean;
  /** 能力（优点）*/
  pros: string[];
  /** 限制（缺点）*/
  cons: string[];
  /** 典型适用场景 */
  bestFor: string;
}

export const TIERS: TierMeta[] = [
  {
    id: "tier1",
    name: "端点映射",
    subtitle: "应用层端点重写 · 无需证书",
    description:
      "AI 工具主动指向 ClawHeart 反向代理端点，仅审计已显式接入的工具流量。HTTPS 流量不解密。",
    icon: Sparkles,
    color: "monitor",
    recommended: true,
    pros: [
      "无需安装证书或修改系统配置",
      "接入与移除均为可逆操作",
      "兼容 MDM 受控的企业终端",
    ],
    cons: [
      "未显式接入的工具不在监控范围内",
      "部分 SDK 硬编码 URL 无法生效",
      "MCP stdio 协议不可达",
    ],
    bestFor: "个人开发者 · 首次接入 · 受控终端",
  },
  {
    id: "tier2",
    name: "系统代理",
    subtitle: "系统级 HTTPS 拦截 · 自签 CA · TLS 解密",
    description:
      "在操作系统层设置代理与自签 CA 形成可信中间人，自动覆盖所有遵循系统代理的应用程序。",
    icon: ShieldCheck,
    color: "scan",
    pros: [
      "自动覆盖所有遵循系统代理的进程",
      "解密 HTTPS 后可进行完整内容审计",
      "一次部署，长期生效",
    ],
    cons: [
      "需要安装并信任 ClawHeart 自签 CA",
      "TLS Pinning 客户端会拒绝连接",
      "增加 ClawHeart 自身的攻击面权重",
    ],
    bestFor: "持续开发使用 · 团队统一审计 · 合规取证",
  },
  {
    id: "tier3",
    name: "沙箱隔离",
    subtitle: "内核级强制 · 进程网络出口约束",
    description:
      "通过 OS 沙箱机制约束目标进程的网络出口，实现进程级 100% 流量覆盖，无旁路。",
    icon: Lock,
    color: "advisory",
    pros: [
      "进程网络出口 100% 覆盖，无旁路",
      "不依赖应用配合或系统代理",
      "可同步约束文件系统访问",
    ],
    cons: [
      "仅适用于通过 sandbox 命令启动的进程",
      "macOS / Linux 实现差异，需平台特定支持",
      "Windows 平台需等待 v2.1 版本",
    ],
    bestFor: "不可信工具评估 · 强制审计 · 合规取证",
  },
];

export function getTier(id: AccessTier): TierMeta {
  return TIERS.find((t) => t.id === id) ?? TIERS[0];
}
