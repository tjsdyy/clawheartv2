import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { TOOLS } from "./tools.config";
import { useStatus } from "@/hooks/useStatus";
import { useAgents } from "@/hooks/useAgents";

/**
 * 首页 Hero — A 流水线 + L0-L7 line chip + 7 stream-pill。
 * 左侧 6 个 Agent 节点根据本机发现状态着色：发现 = 原色 · 未发现 = 灰态。
 */
export function HomeHero() {
  const navigate = useNavigate();
  const { data: status } = useStatus();
  const { data: agents = [] } = useAgents();
  const protectedNow = status?.protected ?? true;

  // 把发现的 Agent 映射到 6 个 slot — 已知 platform 占位优先，未知占空闲
  const slots = useMemo(() => resolveAgentSlots(agents), [agents]);

  return (
    <>
      <div className="hero-pipeline">
        <div className="grid-bg" />
        <div className="glow-center" />

        {/* SVG flow lines */}
        <svg
          style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none", zIndex: 5 }}
          viewBox="0 0 1100 380"
          preserveAspectRatio="none"
        >
          <defs>
            <path id="hh-in-1" d="M 130 60  C 280 60,  400 180, 490 190" />
            <path id="hh-in-2" d="M 80 115  C 240 115, 380 180, 490 190" />
            <path id="hh-in-3" d="M 80 265  C 240 265, 380 200, 490 190" />
            <path id="hh-in-4" d="M 130 320 C 280 320, 400 200, 490 190" />
            <path id="hh-in-5" d="M 80 190  C 220 190, 380 190, 490 190" />
            <path id="hh-in-6" d="M 200 85  C 320 95,  420 180, 490 190" />
            <path id="hh-out-1" d="M 610 190 C 660 190, 700 50,  720 50  L 880 50"  />
            <path id="hh-out-2" d="M 610 190 C 660 190, 700 90,  720 90  L 880 90"  />
            <path id="hh-out-3" d="M 610 190 C 660 190, 700 130, 720 130 L 880 130" />
            <path id="hh-out-4" d="M 610 190 C 660 190, 700 170, 720 170 L 880 170" />
            <path id="hh-out-5" d="M 610 190 C 660 190, 700 210, 720 210 L 880 210" />
            <path id="hh-out-6" d="M 610 190 C 660 190, 700 250, 720 250 L 880 250" />
            <path id="hh-out-7" d="M 610 190 C 660 190, 700 290, 720 290 L 880 290" />
            <path id="hh-out-8" d="M 610 190 C 660 190, 700 330, 720 330 L 880 330" />
          </defs>
          <g stroke="rgb(var(--text-muted))" strokeWidth="1.5" fill="none" className="path-line" opacity="0.4">
            <use href="#hh-in-1" /><use href="#hh-in-2" /><use href="#hh-in-3" />
            <use href="#hh-in-4" /><use href="#hh-in-5" /><use href="#hh-in-6" />
          </g>
          <g stroke="rgb(var(--accent))" strokeWidth="1.5" fill="none" className="path-line" opacity="0.45">
            <use href="#hh-out-1" />
            <use href="#hh-out-2" />
            <use href="#hh-out-3" />
            <use href="#hh-out-4" />
            <use href="#hh-out-5" />
            <use href="#hh-out-6" />
            <use href="#hh-out-7" />
            <use href="#hh-out-8" strokeWidth="2" opacity="1" />
          </g>
          {/* Input particles */}
          {[
            ["#F97316", "2.6s", "#hh-in-1"],
            ["#A855F7", "3.0s", "#hh-in-2"],
            ["#06B6D4", "2.8s", "#hh-in-3"],
            ["#3B82F6", "3.2s", "#hh-in-4"],
            ["#EC4899", "2.5s", "#hh-in-5"],
            ["#4F46E5", "2.9s", "#hh-in-6"],
          ].map(([color, dur, href]) => (
            <g key={href}>
              <circle r="5" fill={color} opacity="0.4" />
              <circle r="2.5" fill={color} />
              <animateMotion dur={dur} repeatCount="indefinite">
                <mpath href={href} />
              </animateMotion>
            </g>
          ))}
          {/* Output particles — 统一 accent 色 */}
          {[
            ["2.0s", "#hh-out-1"],
            ["2.2s", "#hh-out-2"],
            ["2.1s", "#hh-out-3"],
            ["1.9s", "#hh-out-4"],
            ["2.3s", "#hh-out-5"],
            ["2.0s", "#hh-out-6"],
            ["2.4s", "#hh-out-7"],
            ["1.8s", "#hh-out-8"],
          ].map(([dur, href]) => (
            <g key={href}>
              <circle r="5" fill="rgb(var(--accent))" opacity="0.35" />
              <circle r="2.5" fill="rgb(var(--accent))" />
              <animateMotion dur={dur} repeatCount="indefinite">
                <mpath href={href} />
              </animateMotion>
            </g>
          ))}
        </svg>

        {/* Left: 6 AI Agents — 槽位驱动，发现的 active 上色，未发现灰态 */}
        {slots.map((slot, i) => (
          <AgentNode
            key={i}
            left={slot.left}
            top={slot.top}
            color={slot.color}
            label={slot.label}
            active={slot.active}
            title={slot.title}
            duration={slot.duration}
            delay={slot.delay}
            onClick={() => navigate("/tools/agents")}
          >
            <AgentSvg shape={slot.shape} />
          </AgentNode>
        ))}

        {/* Center: ClawHeart logo + 防御层轨道 */}
        <div className="core-cluster">
          <div className="defense-orbit" />
          <div className="defense-orbit-2" />
          <button
            className="core"
            onClick={() => {
              const msg = protectedNow ? "ClawHeart 防护中 · 8 层全部启用" : "防护已暂停";
              toast.info(msg, { description: "拦截事件查看「监控」工具页" });
            }}
          >
            <span className={`core-badge ${protectedNow ? "" : "danger"}`}>
              ● {protectedNow ? "防护中" : "已暂停"}
            </span>
            <svg className="core-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M12 2 L3 6 L3 11 C3 17 7 21 12 22 C17 21 21 17 21 11 L21 6 Z" />
              <path d="M9 12l2 2 4-4" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
            <div className="core-name">ClawHeart</div>
            <div className="core-role">8 层防御</div>
          </button>
        </div>

        {/* L0-L7 chip 贴在 8 条 output 线水平段（chip 居中于 ClawHeart 与 pill 之间） */}
        {LINE_CHIPS.map((chip) => (
          <button
            key={chip.id}
            className="line-chip"
            style={{ left: "66%", top: chip.top }}
            onClick={() => navigate(chip.path)}
            title={chip.tooltip}
          >
            <span className="id">{chip.id}</span>
            <span className="name">{chip.name}</span>
          </button>
        ))}

        {/* Right: 8 stream-pill — 每个 pill 的 top 与对应 chip 完全一致 */}
        <div className="output-stack">
          {STREAM_PILLS.map((p, i) => (
            <button
              key={p.label}
              className={`stream-pill ${p.result ? "result" : ""}`}
              style={{ top: LINE_CHIPS[i].top }}
              onClick={() => navigate(p.path)}
              title={p.tooltip}
            >
              {p.result ? (
                <svg className="check" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              ) : (
                <>
                  <span className="layer-num">{p.layer}</span>
                  <span className="dot" />
                </>
              )}
              <span className="label">{p.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Tools belt — 11 工具入口（接路由） */}
      <div className="hero-tools-belt">
        {TOOLS.filter((t) => t.id !== "more").map((tool) => {
          const Icon = tool.icon;
          const isSoon = tool.status === "coming_soon";
          return (
            <button
              key={tool.id}
              className={`hero-tool-pill ${isSoon ? "soon" : ""}`}
              style={{ ["--tc" as any]: `rgb(var(--tool-${tool.color}))` }}
              onClick={() => !isSoon && navigate(`/tools/${tool.id}`)}
              disabled={isSoon}
            >
              <span className="hero-tool-pill-icon">
                <Icon width={16} height={16} strokeWidth={2} />
              </span>
              <span className="hero-tool-pill-label">{tool.label}</span>
            </button>
          );
        })}
      </div>
    </>
  );
}

interface AgentNodeProps {
  left: string;
  top: string;
  color: string;
  label: string;
  duration: string;
  delay: string;
  active: boolean;
  title: string;
  children: React.ReactNode;
  onClick: () => void;
}
function AgentNode({
  left,
  top,
  color,
  label,
  duration,
  delay,
  active,
  title,
  children,
  onClick,
}: AgentNodeProps) {
  return (
    <button
      className="agent-node"
      style={{
        left, top,
        animation: `hero-float-slow ${duration} ease-in-out ${delay} infinite`,
        opacity: active ? 1 : 0.35,
        filter: active ? undefined : "grayscale(0.7)",
      }}
      onClick={onClick}
      title={title}
    >
      <div
        className="agent-node-icon"
        style={{ color: active ? color : "rgb(var(--text-muted))" }}
      >
        {children}
      </div>
      <div className="agent-node-label">{label}</div>
    </button>
  );
}

// ──────────────────────────────────────────────────────────────────
// Agent slot system —— 6 个固定槽位，按发现状态着色
// ──────────────────────────────────────────────────────────────────
type AgentShape = "claude" | "codex" | "cursor" | "gemini" | "windsurf" | "openclaw" | "generic";

interface AgentSlot {
  /** 槽位的目标 platform 关键字（lowercase 子串匹配） */
  platformKey: string;
  left: string;
  top: string;
  color: string;
  defaultLabel: string;
  shape: AgentShape;
  duration: string;
  delay: string;
  // 运行态字段（由 resolveAgentSlots 填充）
  label: string;
  active: boolean;
  title: string;
}

const SLOT_TEMPLATES: Omit<AgentSlot, "label" | "active" | "title">[] = [
  { platformKey: "claude",   left: "12%", top: "16%", color: "#F97316", defaultLabel: "Claude",   shape: "claude",   duration: "4s",   delay: "0s"   },
  { platformKey: "codex",    left: "8%",  top: "30%", color: "#A855F7", defaultLabel: "Codex",    shape: "codex",    duration: "5s",   delay: "0.1s" },
  { platformKey: "cursor",   left: "8%",  top: "70%", color: "#06B6D4", defaultLabel: "Cursor",   shape: "cursor",   duration: "4.5s", delay: "0.2s" },
  { platformKey: "gemini",   left: "12%", top: "84%", color: "#3B82F6", defaultLabel: "Gemini",   shape: "gemini",   duration: "6s",   delay: "0.3s" },
  { platformKey: "windsurf", left: "6%",  top: "50%", color: "#EC4899", defaultLabel: "Windsurf", shape: "windsurf", duration: "5.5s", delay: "0.4s" },
  { platformKey: "openclaw", left: "18%", top: "22%", color: "#4F46E5", defaultLabel: "OpenClaw", shape: "openclaw", duration: "4.8s", delay: "0.5s" },
];

interface DiscoveredAgentLike {
  platform: string;
  agent_name: string;
}

function resolveAgentSlots(agents: DiscoveredAgentLike[]): AgentSlot[] {
  const slots: AgentSlot[] = SLOT_TEMPLATES.map((t) => ({
    ...t,
    label: t.defaultLabel,
    active: false,
    title: `${t.defaultLabel} · 未发现`,
  }));

  const unmatched: DiscoveredAgentLike[] = [];

  // Pass 1：按 platform 关键字匹配
  for (const a of agents) {
    const plat = (a.platform ?? "").toLowerCase();
    const name = (a.agent_name ?? "").toLowerCase();
    const idx = slots.findIndex(
      (s) =>
        !s.active &&
        (plat.includes(s.platformKey) || name.includes(s.platformKey)),
    );
    if (idx >= 0) {
      slots[idx].label = a.agent_name || slots[idx].defaultLabel;
      slots[idx].active = true;
      slots[idx].title = `${a.agent_name} · ${a.platform} · 已发现`;
    } else {
      unmatched.push(a);
    }
  }

  // Pass 2：未匹配的 Agent 占用空闲槽位（替换 label / 改通用形状）
  for (const a of unmatched) {
    const idx = slots.findIndex((s) => !s.active);
    if (idx < 0) break; // slot 用尽
    slots[idx].label = a.agent_name;
    slots[idx].active = true;
    slots[idx].shape = "generic";
    slots[idx].title = `${a.agent_name} · ${a.platform} · 已发现`;
  }

  return slots;
}

function AgentSvg({ shape }: { shape: AgentShape }) {
  switch (shape) {
    case "claude":
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2L1 21h22L12 2zm0 4l8 14H4L12 6z" />
        </svg>
      );
    case "codex":
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="9" />
          <path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18" />
        </svg>
      );
    case "cursor":
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M4 6h16v2H4V6zm0 5h16v2H4v-2zm0 5h10v2H4v-2z" />
        </svg>
      );
    case "gemini":
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 1l9 4v6c0 5-3.8 9.7-9 11-5.2-1.3-9-6-9-11V5l9-4z" />
        </svg>
      );
    case "windsurf":
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 1v6M12 17v6" stroke="currentColor" strokeWidth="2" fill="none" />
        </svg>
      );
    case "openclaw":
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2z" />
        </svg>
      );
    case "generic":
    default:
      // 通用六角形 — 留给未知 platform
      return (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 2l9 5v10l-9 5-9-5V7z" />
          <circle cx="12" cy="12" r="3" fill="currentColor" />
        </svg>
      );
  }
}

// 8 层 chip + 1 放行 = 9 chip 全部贴在 output 线上
// path 终点 y in 380 viewBox: 50/90/130/170/210/250/290/330（间隔 40）
// chip top%: 50/380 = 13.2% → 86.8%
const LINE_CHIPS = [
  { id: "L0", name: "协议",   top: "13.2%", path: "/tools/logs",     tooltip: "协议归一化 → 请求日志" },
  { id: "L1", name: "网络",   top: "23.7%", path: "/tools/monitor",  tooltip: "网络代理 + 熔断 → 监控" },
  { id: "L2", name: "内容",   top: "34.2%", path: "/tools/monitor",  tooltip: "内容 DLP → 监控" },
  { id: "L3", name: "MCP",   top: "44.7%", path: "/tools/monitor",  tooltip: "MCP 攻击链 → 监控" },
  { id: "L4", name: "供应链", top: "55.3%", path: "/tools/advisory", tooltip: "公告 + 技能签名 → 公告" },
  { id: "L5", name: "系统",   top: "65.8%", path: "/tools/agents",   tooltip: "Agent + 漂移 → Agent 管理" },
  { id: "L6", name: "数据",   top: "76.3%", path: "/tools/budget",   tooltip: "Token 用量 → 预算" },
  { id: "L7", name: "应急",   top: "86.8%", path: "/tools/scan",     tooltip: "Kill Switch + 限速 → 扫描" },
];

const STREAM_PILLS = [
  { layer: "L0", label: "已归一化", path: "/tools/logs",     tooltip: "请求日志",   result: false },
  { layer: "L1", label: "已代理",   path: "/tools/monitor",  tooltip: "MITM 代理",  result: false },
  { layer: "L2", label: "已脱敏",   path: "/tools/monitor",  tooltip: "拦截事件",   result: false },
  { layer: "L3", label: "已查链",   path: "/tools/monitor",  tooltip: "MCP 攻击链", result: false },
  { layer: "L4", label: "已校签",   path: "/tools/advisory", tooltip: "公告校验",   result: false },
  { layer: "L5", label: "已扫描",   path: "/tools/agents",   tooltip: "Agent 发现", result: false },
  { layer: "L6", label: "已记账",   path: "/tools/budget",   tooltip: "Token 用量", result: false },
  { layer: "",   label: "放行 ✓",   path: "/tools/monitor",  tooltip: "最终结果",   result: true  },
];
