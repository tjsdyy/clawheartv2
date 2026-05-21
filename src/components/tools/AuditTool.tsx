import { FileDown, ShieldCheck, AlertTriangle, Loader2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { useStatus } from "@/hooks/useStatus";

const FORMATS = [
  { id: "json", label: "JSON", desc: "原始结构化数据，含 MITRE ATT&CK 映射" },
  { id: "csv", label: "CSV", desc: "便于 Excel / 数据透视分析" },
  { id: "stix", label: "STIX 2.1", desc: "标准威胁情报格式（SIEM 摄取）" },
  { id: "sarif", label: "SARIF", desc: "GitHub / SonarQube 兼容" },
];

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export function AuditTool() {
  const { data: status } = useStatus();
  const [exporting, setExporting] = useState<string | null>(null);

  const stats = [
    {
      label: "总事件",
      value: status?.today_requests?.toLocaleString() ?? "0",
      hint: "今日",
    },
    {
      label: "关键拦截",
      value: status?.today_blocks?.toString() ?? "0",
      hint: status?.today_blocks ? `今日 critical` : "今日",
    },
    {
      label: "MITRE 覆盖",
      value: "21",
      hint: "技术编号 / 内置",
    },
    {
      label: "脱敏比例",
      value: status?.today_requests ? "100%" : "—",
      hint: "类保留 <pl:CLASS:N>",
    },
  ];

  async function handleExport(format: string) {
    setExporting(format);
    try {
      if (inTauri) {
        const path = await invoke<string>("export_request_logs", { format });
        toast.success(`已导出 ${format.toUpperCase()}`, {
          description: path,
          duration: 8000,
        });
      } else {
        toast.info(`浏览器模式：${format} 导出仅 Tauri 中可用`);
      }
    } catch (e) {
      toast.error(`导出失败：${e}`);
    } finally {
      setExporting(null);
    }
  }

  return (
    <div className="mx-auto py-8 px-12" style={{ maxWidth: 880 }}>
      <h2 className="text-[22px] font-semibold tracking-tight mb-1">审计报告导出</h2>
      <p className="text-[13px] text-text-dim mb-6">
        全部拦截事件附 MITRE ATT&CK ID，自动脱敏，开箱即用接入 SIEM
      </p>

      <div className="grid grid-cols-4 gap-3 mb-6">
        {stats.map((s) => (
          <div key={s.label} className="surface p-4">
            <div className="text-[11px] uppercase tracking-wider text-text-muted font-mono">
              {s.label}
            </div>
            <div className="text-[24px] font-semibold mt-1 leading-none tracking-tight">
              {s.value}
            </div>
            <div className="text-[11px] text-text-dim font-mono mt-1.5">{s.hint}</div>
          </div>
        ))}
      </div>

      <h3 className="text-[13px] font-semibold mb-3">选择导出格式</h3>
      <div className="grid grid-cols-2 gap-3 mb-6">
        {FORMATS.map((f) => (
          <button
            key={f.id}
            onClick={() => handleExport(f.id)}
            disabled={exporting !== null}
            className="surface p-4 text-left hover:border-accent transition-colors group disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div className="flex items-center justify-between mb-1.5">
              <div className="font-semibold text-[14px]">{f.label}</div>
              {exporting === f.id ? (
                <Loader2 className="w-4 h-4 animate-spin text-accent" />
              ) : (
                <FileDown className="w-4 h-4 text-text-dim group-hover:text-accent transition-colors" />
              )}
            </div>
            <div className="text-[12px] text-text-dim leading-snug">{f.desc}</div>
          </button>
        ))}
      </div>

      <div className="surface p-4 flex items-start gap-3">
        <div className="text-accent flex-shrink-0 mt-0.5">
          <ShieldCheck className="w-5 h-5" />
        </div>
        <div className="text-[12px] text-text-dim leading-relaxed">
          所有导出都通过类保留脱敏管线，原始凭据 / token / API key 永不出现在报告中。
          所有事件 hash 写入审计链，可用 <code className="bg-bg-elev2 px-1.5 py-0.5 rounded font-mono text-[11px]">clawheart audit verify</code> 命令独立验证。
        </div>
      </div>

      <div className="surface p-4 mt-3 flex items-start gap-3 border-l-[3px]" style={{ borderLeftColor: "rgb(var(--medium))" }}>
        <div className="flex-shrink-0 mt-0.5" style={{ color: "rgb(var(--medium))" }}>
          <AlertTriangle className="w-5 h-5" />
        </div>
        <div className="text-[12px] text-text-dim leading-relaxed">
          当前 W1 alpha 阶段尚未产生真实拦截事件（需启用代理引擎，W5+）。
          导出功能在 W6 完成实现，目前会调用占位 IPC 命令。
        </div>
      </div>
    </div>
  );
}
