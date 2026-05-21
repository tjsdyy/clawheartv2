/**
 * 监控模式页底部：列出所有已发现的 Agent 及其当前所用 Tier。
 * v2.0 共用全局 Tier；"单独覆盖"在 v2.1 实现。
 */
import { useNavigate } from "react-router-dom";
import { ChevronRight, Users } from "lucide-react";
import { useAgents } from "@/hooks/useAgents";
import { getTier } from "./data";
import type { AccessTier } from "@/hooks/useAccessMode";

interface Props {
  currentTier: AccessTier;
}

export function AgentTierOverrideList({ currentTier }: Props) {
  const navigate = useNavigate();
  const { data: agents = [], isLoading } = useAgents();
  const tier = getTier(currentTier);

  // 过滤掉候选 Agent（未确认的不参与流量监控）
  const managed = agents.filter((a) => a.status !== "candidate");

  return (
    <section className="mx-auto max-w-[1100px] mt-8 pt-6 border-t border-border-soft">
      <header className="mb-3">
        <h3 className="text-[13.5px] font-semibold tracking-tight flex items-center gap-2">
          <Users className="w-4 h-4 text-text-muted" />
          Agent 维度覆盖情况
          <span className="text-[11.5px] font-normal text-text-muted">
            · {managed.length} 个 Agent
          </span>
        </h3>
        <p className="text-[11.5px] text-text-muted mt-1 leading-relaxed">
          当前所有 Agent 共用全局监控模式：
          <span
            className="mx-1 font-medium"
            style={{ color: `rgb(var(--tool-${tier.color}))` }}
          >
            {tier.name}
          </span>
          。单 Agent 维度的覆盖将在 v2.1 提供。
        </p>
      </header>

      {isLoading ? (
        <div className="text-[12px] text-text-muted py-4">加载 Agent 列表…</div>
      ) : managed.length === 0 ? (
        <div className="text-[12px] text-text-muted py-4 px-3 border border-border-soft border-dashed rounded-md">
          尚未发现任何已纳入管理的 Agent
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
          {managed.map((a) => {
            const id = `${a.platform}/${a.agent_name}`;
            return (
              <button
                key={id}
                onClick={() => navigate("/tools/agents")}
                className="text-left px-3 py-2 rounded-md border border-border-soft hover:border-text-muted/60 bg-bg-elev transition-colors"
              >
                <div className="flex items-center gap-2 mb-0.5">
                  <span
                    className="w-1.5 h-1.5 rounded-full flex-shrink-0"
                    style={{ background: `rgb(var(--tool-${tier.color}))` }}
                  />
                  <span className="text-[12.5px] font-medium truncate flex-1">
                    {a.agent_name}
                  </span>
                  <span className="text-[10.5px] text-text-muted font-mono">
                    {a.platform.startsWith("unknown:")
                      ? a.platform.slice(8)
                      : a.platform}
                  </span>
                  <ChevronRight className="w-3 h-3 text-text-muted flex-shrink-0" />
                </div>
                <div className="text-[10.5px] text-text-muted flex items-center gap-1.5 ml-3.5">
                  <span>使用</span>
                  <span
                    className="font-medium"
                    style={{ color: `rgb(var(--tool-${tier.color}))` }}
                  >
                    {tier.name}
                  </span>
                  <span>· 跟随全局</span>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </section>
  );
}
