/**
 * 「从 Agent 配置导入渠道」弹层
 *
 * 解析该 Agent 配置文件（如 ~/.openclaw/openclaw.json 的 models.providers.*），
 * 列出候选渠道，用户勾选 → 批量创建 ClawHeart profile + 分配给当前 Agent。
 */
import { useEffect, useState } from "react";
import { X, Loader2, Download, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { DiscoveredAgent } from "@/hooks/useAgents";
import {
  useScanImportableChannels,
  useImportChannelsBatch,
  type ChannelCandidate,
} from "@/hooks/useChannelImport";

interface Props {
  agent: DiscoveredAgent;
  onClose: () => void;
}

const PROTOCOL_LABELS: Record<string, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  openai_responses: "OpenAI Resp.",
  gemini: "Gemini",
};

export function ImportFromAgentDialog({ agent, onClose }: Props) {
  const agentId = `${agent.platform}/${agent.agent_name}`;
  const { data: candidates = [], isLoading } = useScanImportableChannels(agentId);
  const importMutation = useImportChannelsBatch();

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [assignToAgent, setAssignToAgent] = useState(true);

  // 默认勾选所有"未存在"的候选
  useEffect(() => {
    const next = new Set<string>();
    for (const c of candidates) {
      if (!c.already_exists) next.add(c.id);
    }
    setSelected(next);
  }, [candidates.length]); // eslint-disable-line react-hooks/exhaustive-deps

  function toggle(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelected(next);
  }

  function selectAll() {
    setSelected(
      new Set(candidates.filter((c) => !c.already_exists).map((c) => c.id)),
    );
  }

  function clearSelection() {
    setSelected(new Set());
  }

  async function handleImport() {
    const chosen = candidates.filter((c) => selected.has(c.id));
    await importMutation.mutateAsync({
      candidates: chosen,
      assignToAgent: assignToAgent ? agentId : undefined,
    });
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl max-h-[85vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div>
            <h3 className="text-[14px] font-semibold tracking-tight flex items-center gap-2">
              <Download className="w-4 h-4 text-accent" />
              从「{agent.agent_name}」配置导入渠道
            </h3>
            <div className="text-[11.5px] text-text-muted mt-0.5 font-mono">
              {agent.config_path ?? agent.platform} · 发现 {candidates.length} 个候选
            </div>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text">
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* Body */}
        <div className="flex-1 overflow-auto p-5 space-y-3">
          {isLoading ? (
            <div className="flex items-center justify-center py-10 text-text-muted text-[13px]">
              <Loader2 className="w-4 h-4 animate-spin mr-2" />
              解析配置中…
            </div>
          ) : candidates.length === 0 ? (
            <EmptyOrUnsupported agent={agent} />
          ) : (
            <>
              <div className="flex items-center justify-between text-[11.5px] text-text-muted">
                <span>已选 {selected.size} 个</span>
                <div className="flex gap-2">
                  <button
                    onClick={selectAll}
                    className="text-accent hover:underline"
                  >
                    全选未存在
                  </button>
                  <button onClick={clearSelection} className="hover:text-text">
                    清空
                  </button>
                </div>
              </div>

              <div className="space-y-1.5">
                {candidates.map((c) => (
                  <CandidateRow
                    key={c.id}
                    candidate={c}
                    checked={selected.has(c.id)}
                    onToggle={() => toggle(c.id)}
                  />
                ))}
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center gap-3">
          <label className="flex items-center gap-1.5 text-[11.5px] text-text-dim cursor-pointer">
            <input
              type="checkbox"
              checked={assignToAgent}
              onChange={(e) => setAssignToAgent(e.target.checked)}
              className="w-3.5 h-3.5 accent-accent"
            />
            导入后自动分配给「{agent.agent_name}」
          </label>
          <div className="flex-1" />
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
          >
            取消
          </button>
          <button
            onClick={handleImport}
            disabled={selected.size === 0 || importMutation.isPending}
            className="flex items-center gap-1.5 px-4 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
          >
            {importMutation.isPending ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Download className="w-3.5 h-3.5" />
            )}
            导入 {selected.size > 0 && selected.size}
          </button>
        </footer>
      </div>
    </div>
  );
}

function CandidateRow({
  candidate,
  checked,
  onToggle,
}: {
  candidate: ChannelCandidate;
  checked: boolean;
  onToggle: () => void;
}) {
  const host = (() => {
    try {
      return new URL(candidate.base_url).host;
    } catch {
      return candidate.base_url;
    }
  })();

  return (
    <label
      className={cn(
        "flex items-center gap-3 px-3 py-2 rounded-md border bg-bg-elev cursor-pointer transition-colors",
        checked
          ? "border-accent/40 bg-accent/[0.04]"
          : "border-border-soft hover:border-text-muted/60",
        candidate.already_exists && "opacity-60 cursor-not-allowed",
      )}
    >
      <input
        type="checkbox"
        checked={checked}
        disabled={candidate.already_exists}
        onChange={onToggle}
        className="w-3.5 h-3.5 accent-accent flex-shrink-0"
      />
      <div
        className="w-7 h-7 rounded flex items-center justify-center flex-shrink-0 font-mono text-[10px] font-bold bg-bg-elev2 text-text-muted"
      >
        {candidate.name.charAt(0).toUpperCase()}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-0.5 flex-wrap">
          <span className="text-[12.5px] font-medium truncate">
            {candidate.name}
          </span>
          <span className="text-[9.5px] font-mono uppercase tracking-wider px-1.5 py-0.5 rounded bg-bg-elev2 text-text-muted">
            {PROTOCOL_LABELS[candidate.protocol] ?? candidate.protocol}
          </span>
          {candidate.default_model && (
            <span className="text-[10px] px-1 py-0.5 rounded text-text-muted bg-bg-elev2 font-mono">
              {candidate.default_model}
            </span>
          )}
          {candidate.already_exists && (
            <span className="text-[10px] px-1.5 py-0.5 rounded text-text-muted bg-bg-elev2">
              已存在
            </span>
          )}
          {!candidate.api_key && (
            <span className="text-[10px] px-1.5 py-0.5 rounded text-amber-600 dark:text-amber-400 bg-amber-500/10">
              无凭据
            </span>
          )}
        </div>
        <div className="text-[10.5px] text-text-muted font-mono truncate">
          {host}
        </div>
        {candidate.warnings.length > 0 && (
          <div className="mt-1 flex items-start gap-1 text-[10.5px] text-amber-600 dark:text-amber-400">
            <AlertTriangle className="w-2.5 h-2.5 mt-0.5 flex-shrink-0" />
            <span>{candidate.warnings.join("；")}</span>
          </div>
        )}
      </div>
    </label>
  );
}

// ──────────────────────────────────────────────────────────────────
// 空状态 / 不支持平台 提示
// ──────────────────────────────────────────────────────────────────

/** 已实现反向导入的平台 + 对应配置文件路径（用于"未找到/不支持"时的提示） */
const PLATFORM_CONFIG_HINT: Record<
  string,
  { supported: boolean; configPath: string; hint: string }
> = {
  openclaw: {
    supported: true,
    configPath: "~/.openclaw/openclaw.json",
    hint: "检查 models.providers.{} 字段是否有 provider 条目",
  },
  openeva: {
    supported: true,
    configPath: "~/.openeva/tool-providers.json 或 settings.json",
    hint: "检查 providers / llm.providers / tools.providers 字段",
  },
  claude: {
    supported: true,
    configPath: "~/.claude/settings.json",
    hint: "检查 env.ANTHROPIC_BASE_URL + env.ANTHROPIC_AUTH_TOKEN",
  },
  codex: {
    supported: true,
    configPath: "~/.codex/config.toml + ~/.codex/auth.json",
    hint: "检查 [model_providers.*] section 和 OPENAI_API_KEY",
  },
  gemini: {
    supported: true,
    configPath: "~/.gemini/.env + ~/.gemini/settings.json",
    hint: "检查 GEMINI_API_KEY；OAuth 登录用户需手动补 API key",
  },
  opencode: {
    supported: true,
    configPath: "~/.config/opencode/opencode.json",
    hint: "检查 provider.<id>.options.{baseURL, apiKey} 字段",
  },
  hermes: {
    supported: true,
    configPath: "~/.hermes/config.yaml + ~/.hermes/.env",
    hint: "检查 custom_providers[] 数组（含 name/base_url/api_key/model）",
  },
};

function EmptyOrUnsupported({ agent }: { agent: { platform: string; config_path: string | null } }) {
  const info = PLATFORM_CONFIG_HINT[agent.platform];
  const supported = info?.supported ?? false;

  if (supported) {
    // 支持但本次扫不出来 —— 显示文件路径提示用户检查
    return (
      <div className="px-6 py-10 text-center border border-dashed border-border-soft rounded-md">
        <div className="text-[13px] text-text-muted mb-2">
          未在该 Agent 配置中发现可导入的渠道
        </div>
        <div className="text-[11.5px] text-text-muted leading-relaxed">
          预期文件：
          <code className="font-mono px-1.5 py-0.5 rounded bg-bg-elev2 text-text-dim ml-1">
            {info.configPath}
          </code>
          <br />
          {info.hint}
        </div>
      </div>
    );
  }

  // 不支持的平台 —— 明确告知 + 引导手动添加
  return (
    <div className="px-6 py-8 text-center border border-amber-500/30 bg-amber-500/[0.04] rounded-md">
      <div className="text-[13px] font-medium text-amber-700 dark:text-amber-300 mb-2">
        无法自动识别 {agent.platform} 的配置文件
      </div>
      <div className="text-[11.5px] text-text-dim leading-relaxed mb-3">
        该 Agent 的配置文件格式 ClawHeart 暂未做反向解析。
        <br />
        请关闭此弹窗后：
        <ol className="text-left mt-2 ml-4 list-decimal space-y-1">
          <li>
            打开 Agent 的配置文件
            {agent.config_path && (
              <>
                {" "}
                <code className="font-mono px-1 py-0.5 rounded bg-bg-elev2 text-[10px]">
                  {agent.config_path}
                </code>
              </>
            )}
          </li>
          <li>
            手动查看其中已配置的 LLM provider 信息
          </li>
          <li>
            在 ClawHeart 工具栏点击「+ 新建」按钮，按预设填入 base_url 和 API key
          </li>
        </ol>
      </div>
      <div className="text-[10.5px] text-text-muted mt-3 pt-2 border-t border-amber-500/20">
        当前支持自动导入的平台：OpenClaw / OpenEva / OpenCode / Hermes / Claude / Codex / Gemini
      </div>
    </div>
  );
}
