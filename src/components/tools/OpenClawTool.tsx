import { Puzzle, Download, Power, ExternalLink } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

const INTEGRATIONS: Integration[] = [
  { id: "gateway", name: "OpenClaw Gateway", status: "not_detected", desc: "本机网关，统一管理所有 AI Agent" },
  { id: "workspace", name: "OpenClaw Workspace", status: "not_detected", desc: "可视化工作区，集成多 Agent 协同" },
  { id: "audit_legacy", name: "OpenClaw Security Audit (CLI)", status: "embedded", desc: "80 项离线扫描，已收编进「扫描」工具" },
];

type IntegrationStatus = "active" | "stopped" | "not_detected" | "embedded";

interface Integration {
  id: string;
  name: string;
  status: IntegrationStatus;
  desc: string;
}

export function OpenClawTool() {
  const [endpoint, setEndpoint] = useState("");

  function handleConnect() {
    if (!endpoint.trim()) {
      toast.error("请填写 OpenClaw 实例地址");
      return;
    }
    toast.info("OpenClaw 集成 IPC 将在 W19 实现", {
      description: `目标 endpoint：${endpoint}`,
    });
  }

  return (
    <div className="mx-auto py-8 px-12" style={{ maxWidth: 880 }}>
      <div className="flex items-center gap-3 mb-2">
        <div
          className="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0"
          style={{
            background: "color-mix(in srgb, rgb(var(--tool-openclaw)) 12%, transparent)",
            color: "rgb(var(--tool-openclaw))",
          }}
        >
          <Puzzle className="w-6 h-6" />
        </div>
        <div>
          <h2 className="text-[22px] font-semibold tracking-tight">OpenClaw 集成</h2>
          <p className="text-[13px] text-text-dim mt-0.5">
            OpenClaw 套件可选挂载 —— Core-first，下载 / 启停 / 指向已有安装
          </p>
        </div>
      </div>

      <div className="surface p-4 mt-6 flex items-start gap-3 border-l-[3px]" style={{ borderLeftColor: "rgb(var(--tool-openclaw))" }}>
        <Puzzle className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: "rgb(var(--tool-openclaw))" }} />
        <div className="text-[12.5px] text-text-dim leading-relaxed">
          v2 起 OpenClaw 不再默认内置 —— ClawHeart 是<strong className="text-text">本机 AI 安全运行时</strong>，
          OpenClaw 是<strong className="text-text">可选挂载</strong>。
          完整 IPC 集成在 W19 实现；当前页面展示发现状态。
        </div>
      </div>

      <h3 className="text-[13px] font-semibold mt-6 mb-3">可用集成</h3>
      <div className="space-y-2.5">
        {INTEGRATIONS.map((int) => (
          <IntegrationRow key={int.id} integration={int} />
        ))}
      </div>

      <h3 className="text-[13px] font-semibold mt-8 mb-3">指向已有 OpenClaw 安装</h3>
      <div className="surface p-4">
        <div className="text-[12px] text-text-dim mb-3">
          如果你已经有运行中的 OpenClaw 实例，填写其地址：
        </div>
        <div className="flex gap-2">
          <input
            value={endpoint}
            onChange={(e) => setEndpoint(e.target.value)}
            placeholder="http://127.0.0.1:21000"
            className="flex-1 bg-bg border border-border-soft rounded-lg px-3 py-1.5 text-[13px] outline-none focus:border-accent font-mono"
          />
          <button onClick={handleConnect} className="btn-ghost text-[12px]">
            <ExternalLink className="w-3.5 h-3.5" />
            连接
          </button>
        </div>
      </div>
    </div>
  );
}

function IntegrationRow({ integration }: { integration: Integration }) {
  return (
    <div className="surface p-4 flex items-center gap-4">
      <div
        className="w-10 h-10 rounded-lg flex items-center justify-center bg-bg-elev2 border border-border-soft flex-shrink-0"
        style={{ color: "rgb(var(--tool-openclaw))" }}
      >
        <Puzzle className="w-5 h-5" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="font-semibold text-[14px]">{integration.name}</div>
        <div className="text-[12px] text-text-dim mt-0.5">{integration.desc}</div>
      </div>

      <div className="flex-shrink-0">
        {integration.status === "active" && (
          <div className="flex items-center gap-3">
            <span className="chip" style={{ background: "rgb(var(--accent) / 0.1)", color: "rgb(var(--accent))" }}>
              ● 运行中
            </span>
            <button
              onClick={() => toast.info("W19 实现")}
              className="btn-ghost text-[12px]"
            >
              <Power className="w-3.5 h-3.5" />
              停止
            </button>
          </div>
        )}
        {integration.status === "stopped" && (
          <button
            onClick={() => toast.info("W19 实现")}
            className="btn-ghost text-[12px]"
          >
            <Download className="w-3.5 h-3.5" />
            启动
          </button>
        )}
        {integration.status === "not_detected" && (
          <span className="chip bg-bg-elev2 text-text-muted">未检测到</span>
        )}
        {integration.status === "embedded" && (
          <span className="chip bg-bg-elev2 text-text-muted">已收编</span>
        )}
      </div>
    </div>
  );
}
