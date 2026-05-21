import { TOOLS } from "../grid/tools.config";
import { Hourglass, Bell } from "lucide-react";
import { toast } from "sonner";

const REPO_URL = "https://github.com/clawheart/desktop";

export function PlaceholderTool({ toolId }: { toolId: string }) {
  const tool = TOOLS.find((t) => t.id === toolId);
  if (!tool) return null;
  const Icon = tool.icon;
  return (
    <div className="flex flex-col items-center justify-center h-full px-12 py-16 text-center">
      <div
        className="w-16 h-16 rounded-2xl flex items-center justify-center mb-6"
        style={{
          background: `color-mix(in srgb, rgb(var(--tool-${tool.color})) 12%, transparent)`,
          color: `rgb(var(--tool-${tool.color}))`,
        }}
      >
        <Icon className="w-8 h-8" strokeWidth={2} />
      </div>
      <h2 className="text-xl font-semibold mb-2">{tool.label}</h2>
      <p className="text-text-dim text-sm max-w-md mb-8 leading-relaxed">
        {tool.description}
      </p>
      <p className="text-text-muted text-[12px] font-mono mb-6">
        此工具的 UI 在 W13–W16 阶段完整实现。当前为占位页。
      </p>
      <button
        onClick={() => {
          navigator.clipboard.writeText(REPO_URL);
          toast.success("仓库地址已复制", { description: REPO_URL });
        }}
        className="btn-ghost"
      >
        <Bell className="w-4 h-4" />
        关注开发进展
      </button>
    </div>
  );
}

export function SoonTool({ toolId, version }: { toolId: string; version: string }) {
  const tool = TOOLS.find((t) => t.id === toolId);
  if (!tool) return null;
  const Icon = tool.icon;
  return (
    <div className="flex flex-col items-center justify-center h-full px-12 py-16 text-center">
      <div
        className="w-20 h-20 rounded-2xl flex items-center justify-center mb-6 relative"
        style={{
          background: `color-mix(in srgb, rgb(var(--tool-${tool.color})) 12%, transparent)`,
          color: `rgb(var(--tool-${tool.color}))`,
        }}
      >
        <Icon className="w-10 h-10" strokeWidth={1.8} />
        <span className="absolute -top-2 -right-2 chip bg-bg-elev2 border border-border text-text-muted">
          {version}
        </span>
      </div>

      <h2 className="text-2xl font-semibold mb-2 tracking-tight">{tool.label}</h2>
      <p className="text-text-dim text-sm max-w-md mb-8 leading-relaxed">
        {soonDescriptions[toolId] ?? tool.description}
      </p>

      <button
        onClick={() => toast.success(`${tool.label} 上线时会通过更新器告知`, {
          description: `预计 ${version}`,
        })}
        className="btn-primary"
      >
        <Hourglass className="w-4 h-4" />
        上线后通知我
      </button>
      <p className="text-[11px] text-text-muted mt-4 font-mono">即将上线 · {version}</p>
    </div>
  );
}

const soonDescriptions: Record<string, string> = {
  token_verify:
    "校验你使用的 LLM API Token 是否经过未公开的中转站，识别可能的中间人 / 配额劫持 / 模型偷换。比对 TLS 证书链、通过哨兵问题指纹识别模型、探测延迟异常。",
  relay:
    "可选私有代理路由层。把流量定向到自己的中转端点，绕开网络限制，同时保留 ClawHeart 全部安全管线。",
  policy:
    "企业级 / 团队级共享策略下发。集中管理多台机器的规则、技能白名单与预算上限。",
};
