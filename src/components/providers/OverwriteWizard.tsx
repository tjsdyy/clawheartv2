import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  X,
  ArrowRight,
  ArrowLeft,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Loader2,
  ShieldAlert,
  Sparkles,
  Lock,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { toast } from "sonner";
import {
  useProviderProfiles,
  type ProviderProfile,
} from "@/hooks/useProviders";
import {
  useScanAgentConfigs,
  usePlanOverwrite,
  useApplyOverwrite,
  useApplyRealStatus,
  type ConfigPatch,
  type ProbeResult,
  type ApplyBatchResult,
  type PatchRisk,
} from "@/hooks/useAgentConfig";

interface Props {
  initialProfileId?: string;
  /** 锁定单个 Agent（来自 AgentDetail），传入后跳过 Step 2（选 Agent） */
  initialAgentId?: string;
  onClose: () => void;
  onApplied?: (result: ApplyBatchResult) => void;
}

type Step = 1 | 2 | 3 | 4;

export function OverwriteWizard({
  initialProfileId,
  initialAgentId,
  onClose,
  onApplied,
}: Props) {
  const agentLocked = !!initialAgentId;
  const [step, setStep] = useState<Step>(1);
  const [profileId, setProfileId] = useState<string>(initialProfileId ?? "");
  const [selectedAgentIds, setSelectedAgentIds] = useState<Set<string>>(
    agentLocked && initialAgentId ? new Set([initialAgentId]) : new Set(),
  );
  const [patches, setPatches] = useState<ConfigPatch[]>([]);
  const [dryRun, setDryRun] = useState(true);
  const [applyResult, setApplyResult] = useState<ApplyBatchResult | null>(null);

  const { data: profiles = [] } = useProviderProfiles();
  const { data: probes = [], isLoading: scanning } = useScanAgentConfigs();
  const { data: applyReal } = useApplyRealStatus();
  const planMutation = usePlanOverwrite();
  const applyMutation = useApplyOverwrite();

  // 当全局未开启实际写入时，强制 dry-run 不可关闭
  const dryRunLocked = !(applyReal?.enabled ?? false);
  useEffect(() => {
    if (dryRunLocked && !dryRun) setDryRun(true);
  }, [dryRunLocked, dryRun]);

  // 默认选中可接管的全部 Agent（仅非锁定模式）
  useEffect(() => {
    if (agentLocked) return;
    if (probes.length > 0 && selectedAgentIds.size === 0) {
      const auto = new Set(
        probes
          .filter((p) => p.probe_available)
          .map((p) => p.agent_id),
      );
      setSelectedAgentIds(auto);
    }
  }, [agentLocked, probes, selectedAgentIds.size]);

  const selectedProfile = profiles.find((p) => p.id === profileId) ?? null;
  const lockedProbe = agentLocked
    ? probes.find((p) => p.agent_id === initialAgentId) ?? null
    : null;

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-6 animate-fadein">
      <div className="w-full max-w-3xl max-h-[90vh] bg-bg rounded-xl shadow-2xl border border-border flex flex-col overflow-hidden">
        {/* Header */}
        <header className="flex items-center justify-between px-5 py-3.5 border-b border-border">
          <div className="flex items-center gap-2.5">
            <Sparkles className="w-4 h-4 text-accent" />
            <h3 className="text-[14px] font-semibold tracking-tight">
              一键覆盖 Agent 模型配置
            </h3>
          </div>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text"
          >
            <X className="w-4 h-4" />
          </button>
        </header>

        {/* 锁定 Agent 横幅 */}
        {agentLocked && (
          <div className="px-5 py-2 border-b border-border-soft bg-accent/5 flex items-center gap-2 text-[11.5px]">
            <Lock className="w-3 h-3 text-accent" />
            <span className="text-text-dim">
              目标 Agent：
              <strong className="text-text">
                {lockedProbe?.agent_name ?? initialAgentId}
              </strong>
              {lockedProbe && (
                <span className="text-text-muted ml-1.5">
                  ({lockedProbe.agent_platform})
                </span>
              )}
            </span>
          </div>
        )}

        {/* Step indicator */}
        <StepIndicator step={step} agentLocked={agentLocked} />

        {/* Step content */}
        <div className="flex-1 overflow-auto px-6 py-5">
          {step === 1 && (
            <Step1SelectProfile
              profiles={profiles}
              selectedId={profileId}
              onSelect={setProfileId}
            />
          )}
          {step === 2 && !agentLocked && (
            <Step2SelectAgents
              probes={probes}
              scanning={scanning}
              selectedIds={selectedAgentIds}
              onToggle={(id) => {
                const next = new Set(selectedAgentIds);
                if (next.has(id)) next.delete(id);
                else next.add(id);
                setSelectedAgentIds(next);
              }}
              onToggleAll={(checked) => {
                if (checked) {
                  setSelectedAgentIds(
                    new Set(
                      probes
                        .filter((p) => p.probe_available)
                        .map((p) => p.agent_id),
                    ),
                  );
                } else {
                  setSelectedAgentIds(new Set());
                }
              }}
            />
          )}
          {step === 3 && (
            <Step3Diff
              patches={patches}
              selectedProfile={selectedProfile}
              dryRun={dryRun}
              setDryRun={setDryRun}
              dryRunLocked={dryRunLocked}
            />
          )}
          {step === 4 && applyResult && <Step4Result result={applyResult} />}
        </div>

        {/* Footer */}
        <footer className="px-5 py-3 border-t border-border bg-bg-elev/50 flex items-center justify-between">
          <div className="text-[11px] text-text-muted">
            {step < 4 && profileId && selectedProfile && (
              <>
                渠道：<strong className="text-text-dim">{selectedProfile.name}</strong>
                {selectedAgentIds.size > 0 && step >= 2 && (
                  <> · {selectedAgentIds.size} Agent</>
                )}
              </>
            )}
          </div>
          <div className="flex items-center gap-2">
            {step > 1 && step < 4 && (
              <button
                onClick={() => {
                  // 锁定模式：Step 3 → Step 1（跳过 Step 2）
                  if (agentLocked && step === 3) {
                    setStep(1);
                  } else {
                    setStep((s) => (s - 1) as Step);
                  }
                }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12.5px] text-text-dim hover:text-text hover:bg-bg-elev2"
              >
                <ArrowLeft className="w-3.5 h-3.5" />
                上一步
              </button>
            )}
            {step === 1 && (
              <button
                onClick={async () => {
                  if (agentLocked) {
                    // 锁定模式：直接 plan + 跳到 Step 3
                    const result = await planMutation.mutateAsync({
                      profileId,
                      agentIds: Array.from(selectedAgentIds),
                    });
                    setPatches(result);
                    setStep(3);
                  } else {
                    setStep(2);
                  }
                }}
                disabled={!profileId || (agentLocked && planMutation.isPending)}
                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
              >
                {agentLocked && planMutation.isPending && (
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                )}
                {agentLocked ? "下一步：预览变更" : "下一步：选择 Agent"}
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            )}
            {step === 2 && (
              <button
                onClick={async () => {
                  const result = await planMutation.mutateAsync({
                    profileId,
                    agentIds: Array.from(selectedAgentIds),
                  });
                  setPatches(result);
                  setStep(3);
                }}
                disabled={selectedAgentIds.size === 0 || planMutation.isPending}
                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
              >
                {planMutation.isPending && (
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                )}
                计算变更预览
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            )}
            {step === 3 && (
              <button
                onClick={async () => {
                  const r = await applyMutation.mutateAsync({
                    profileId,
                    patches,
                    dryRun,
                  });
                  setApplyResult(r);
                  setStep(4);
                  onApplied?.(r);
                  if (r.failure_count === 0) {
                    toast.success(
                      `${dryRun ? "[Dry-run] " : ""}已应用 ${r.success_count} 处变更`,
                    );
                  } else {
                    toast.warning(
                      `${r.success_count} 成功 / ${r.failure_count} 失败`,
                    );
                  }
                }}
                disabled={patches.length === 0 || applyMutation.isPending}
                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90 disabled:opacity-50"
              >
                {applyMutation.isPending && (
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                )}
                应用 {patches.length} 处变更
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            )}
            {step === 4 && (
              <button
                onClick={onClose}
                className="px-3.5 py-1.5 rounded-md bg-accent text-white text-[12.5px] font-medium hover:bg-accent/90"
              >
                完成
              </button>
            )}
          </div>
        </footer>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Step indicator
// ──────────────────────────────────────────────────────────────────
function StepIndicator({
  step,
  agentLocked,
}: {
  step: Step;
  agentLocked?: boolean;
}) {
  // 锁定模式下跳过「选 Agent」步骤：内部 step 1/3/4 映射到外部 1/2/3
  const steps = agentLocked
    ? ["选渠道", "预览变更", "应用结果"]
    : ["选渠道", "选 Agent", "预览变更", "应用结果"];
  // 把内部 step 折算为外部展示索引
  const displayStep: number = agentLocked
    ? step === 1
      ? 1
      : step === 3
        ? 2
        : step === 4
          ? 3
          : 1
    : step;
  return (
    <div className="flex items-center gap-1 px-5 py-2.5 border-b border-border-soft bg-bg-elev/30">
      {steps.map((label, i) => {
        const n = i + 1;
        const active = n === displayStep;
        const done = n < displayStep;
        return (
          <div key={label} className="flex items-center gap-1">
            <span
              className={cn(
                "w-5 h-5 rounded-full text-[10.5px] flex items-center justify-center font-medium",
                done
                  ? "bg-emerald-500/15 text-emerald-500"
                  : active
                    ? "bg-accent text-white"
                    : "bg-bg-elev2 text-text-muted",
              )}
            >
              {done ? "✓" : n}
            </span>
            <span
              className={cn(
                "text-[11.5px]",
                active ? "text-text font-medium" : "text-text-muted",
              )}
            >
              {label}
            </span>
            {i < steps.length - 1 && (
              <span className="mx-1.5 text-text-muted">›</span>
            )}
          </div>
        );
      })}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Step 1：选 Profile
// ──────────────────────────────────────────────────────────────────
function Step1SelectProfile({
  profiles,
  selectedId,
  onSelect,
}: {
  profiles: ProviderProfile[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  return (
    <div>
      <h4 className="text-[13.5px] font-medium mb-1">选择要应用的渠道</h4>
      <p className="text-[12px] text-text-dim mb-4 leading-relaxed">
        所选渠道的虚拟 key 将被写入各 Agent；真实 API key 不暴露给 Agent。
      </p>
      {profiles.length === 0 ? (
        <div className="text-[12.5px] text-text-muted px-4 py-6 text-center border border-border-soft rounded-md">
          暂无模型渠道，请先在「模型管理」工具页创建一个。
        </div>
      ) : (
        <div className="space-y-1.5">
          {profiles.map((p) => (
            <button
              key={p.id}
              onClick={() => onSelect(p.id)}
              className={cn(
                "w-full flex items-center gap-3 px-3 py-2.5 rounded-md border text-left transition-colors",
                selectedId === p.id
                  ? "border-accent bg-accent/5"
                  : "border-border hover:border-text-muted",
              )}
            >
              <input
                type="radio"
                checked={selectedId === p.id}
                onChange={() => onSelect(p.id)}
                className="w-3.5 h-3.5 accent-accent"
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-1.5 text-[13px] font-medium">
                  {p.name}
                  {p.is_default && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300">
                      默认
                    </span>
                  )}
                  {!p.credential_set && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300">
                      未设凭据
                    </span>
                  )}
                </div>
                <div className="text-[11px] text-text-muted font-mono truncate">
                  {p.base_url}
                </div>
              </div>
              <div className="text-[10.5px] text-text-muted font-mono">
                {p.virtual_key.slice(0, 16)}…
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Step 2：选 Agent
// ──────────────────────────────────────────────────────────────────
function Step2SelectAgents({
  probes,
  scanning,
  selectedIds,
  onToggle,
  onToggleAll,
}: {
  probes: ProbeResult[];
  scanning: boolean;
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
  onToggleAll: (checked: boolean) => void;
}) {
  const supported = probes.filter((p) => p.probe_available);
  const allChecked =
    supported.length > 0 &&
    supported.every((p) => selectedIds.has(p.agent_id));

  return (
    <div>
      <h4 className="text-[13.5px] font-medium mb-1">选择要接管的 Agent</h4>
      <p className="text-[12px] text-text-dim mb-4 leading-relaxed">
        ClawHeart 已发现以下 Agent。仅可接管"支持 Probe"的项；不支持的会显示为只读。
      </p>

      {scanning && (
        <div className="flex items-center gap-2 text-[12px] text-text-muted py-2">
          <Loader2 className="w-3.5 h-3.5 animate-spin" />
          探测中…
        </div>
      )}

      <div className="border border-border-soft rounded-md overflow-hidden">
        <div className="flex items-center gap-2 px-3 py-2 bg-bg-elev border-b border-border-soft">
          <input
            type="checkbox"
            checked={allChecked}
            onChange={(e) => onToggleAll(e.target.checked)}
            className="w-3.5 h-3.5 accent-accent"
            disabled={supported.length === 0}
          />
          <span className="text-[11.5px] text-text-muted flex-1">
            Agent · 当前 base_url · 风险
          </span>
        </div>

        {probes.length === 0 && !scanning && (
          <div className="px-4 py-6 text-center text-[12.5px] text-text-muted">
            未发现可接管的 Agent
          </div>
        )}

        {probes.map((p) => {
          const checked = selectedIds.has(p.agent_id);
          const supported = p.probe_available;
          return (
            <label
              key={p.agent_id}
              className={cn(
                "flex items-center gap-2 px-3 py-2 border-t border-border-soft text-[12px]",
                supported
                  ? "cursor-pointer hover:bg-bg-elev/50"
                  : "opacity-60 cursor-not-allowed",
              )}
            >
              <input
                type="checkbox"
                checked={checked}
                disabled={!supported}
                onChange={() => onToggle(p.agent_id)}
                className="w-3.5 h-3.5 accent-accent"
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-1.5 mb-0.5">
                  <span className="font-medium text-text">{p.agent_name}</span>
                  <span className="text-[10.5px] text-text-muted">
                    {platformLabel(p.agent_platform)}
                  </span>
                  {!supported && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300">
                      W8 接入
                    </span>
                  )}
                </div>
                <div className="text-[10.5px] font-mono text-text-muted truncate">
                  {p.current_base_url ?? "(未设置)"}
                  {p.current_key_present && " · 凭据已设"}
                </div>
                {p.warnings.length > 0 && (
                  <div className="flex items-start gap-1 mt-1 text-[10.5px] text-amber-600 dark:text-amber-400">
                    <AlertTriangle className="w-2.5 h-2.5 mt-0.5 flex-shrink-0" />
                    <span>{p.warnings[0]}</span>
                  </div>
                )}
              </div>
              {supported && <RiskBadge risk="Safe" />}
            </label>
          );
        })}
      </div>
    </div>
  );
}

function RiskBadge({ risk }: { risk: PatchRisk }) {
  const map: Record<
    PatchRisk,
    { label: string; cls: string }
  > = {
    Safe: {
      label: "Safe",
      cls: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
    },
    Caution: {
      label: "Caution",
      cls: "bg-amber-500/15 text-amber-700 dark:text-amber-300",
    },
    Risky: {
      label: "Risky",
      cls: "bg-critical/15 text-critical",
    },
  };
  const { label, cls } = map[risk];
  return (
    <span className={cn("text-[10px] px-1.5 py-0.5 rounded font-medium", cls)}>
      {label}
    </span>
  );
}

// ──────────────────────────────────────────────────────────────────
// Step 3：Diff 预览
// ──────────────────────────────────────────────────────────────────
function Step3Diff({
  patches,
  selectedProfile,
  dryRun,
  setDryRun,
  dryRunLocked,
}: {
  patches: ConfigPatch[];
  selectedProfile: ProviderProfile | null;
  dryRun: boolean;
  setDryRun: (v: boolean) => void;
  dryRunLocked: boolean;
}) {
  const navigate = useNavigate();
  return (
    <div>
      <h4 className="text-[13.5px] font-medium mb-1">变更预览</h4>
      <p className="text-[12px] text-text-dim mb-3 leading-relaxed">
        以下变更将写入各 Agent 配置；
        {dryRun ? (
          <strong className="text-amber-600 dark:text-amber-400">
            {" "}dry-run 模式：仅写入 ~/.clawheart-v2/dry-run/ 沙箱目录
          </strong>
        ) : (
          <strong className="text-critical"> 真实模式：将修改 Agent 配置文件</strong>
        )}
      </p>

      {/* Dry-run 开关 */}
      <div
        className={cn(
          "flex items-center justify-between px-3 py-2.5 mb-3 rounded-md border",
          dryRunLocked
            ? "border-border bg-bg-elev/30"
            : "border-amber-500/30 bg-amber-500/5",
        )}
      >
        <div className="flex items-center gap-2 flex-1 min-w-0">
          {dryRunLocked ? (
            <Lock className="w-4 h-4 text-text-muted flex-shrink-0" />
          ) : (
            <ShieldAlert className="w-4 h-4 text-amber-500 flex-shrink-0" />
          )}
          <div className="text-[12px] leading-snug min-w-0">
            <strong
              className={
                dryRunLocked
                  ? "text-text-dim"
                  : "text-amber-700 dark:text-amber-300"
              }
            >
              Dry-run 沙箱
            </strong>
            <span className="text-text-dim">
              {dryRunLocked
                ? " — 已锁定：未在「设置 → 安全」中启用实际写入"
                : " — 关闭后将直接修改真实文件"}
            </span>
          </div>
        </div>
        {dryRunLocked ? (
          <button
            onClick={() => navigate("/tools/settings")}
            className="flex-shrink-0 text-[11px] px-2 py-1 rounded border border-border hover:border-text-muted text-text-dim hover:text-text"
          >
            打开设置 →
          </button>
        ) : (
          <button
            onClick={() => setDryRun(!dryRun)}
            className={cn(
              "w-10 h-5 rounded-full relative transition-colors flex-shrink-0",
              dryRun ? "bg-amber-500" : "bg-bg-elev2 border border-border",
            )}
          >
            <span
              className={cn(
                "absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-transform",
                dryRun ? "translate-x-[22px]" : "translate-x-0.5",
              )}
            />
          </button>
        )}
      </div>

      {patches.length === 0 ? (
        <div className="text-[12.5px] text-text-muted px-4 py-6 text-center border border-border-soft rounded-md">
          没有可应用的变更
        </div>
      ) : (
        <div className="space-y-3">
          {patches.map((patch) => (
            <PatchCard key={patch.agent_id} patch={patch} />
          ))}
        </div>
      )}
    </div>
  );
}

function PatchCard({ patch }: { patch: ConfigPatch }) {
  const [expanded, setExpanded] = useState(false);
  const previewLines = patch.diff_lines.slice(0, 6);
  const hasMore = patch.diff_lines.length > previewLines.length;

  return (
    <div className="rounded-md border border-border-soft overflow-hidden">
      <div className="flex items-center gap-2 px-3 py-2 bg-bg-elev">
        <span className="text-[13px] font-medium">{patch.agent_name}</span>
        <span className="text-[10.5px] text-text-muted">
          {platformLabel(patch.agent_platform)}
        </span>
        <RiskBadge risk={patch.risk_level} />
        <span className="flex-1 text-[10.5px] font-mono text-text-muted truncate">
          {sourceLabel(patch.source)}
        </span>
        <button
          onClick={() => setExpanded((v) => !v)}
          className="text-[11px] text-text-muted hover:text-text"
        >
          {expanded ? "收起" : "展开"}
        </button>
      </div>
      <div className="bg-bg-elev/40 font-mono text-[10.5px] leading-snug overflow-x-auto">
        {(expanded ? patch.diff_lines : previewLines).map((line, i) => (
          <div
            key={i}
            className={cn(
              "px-3 py-0.5",
              line.kind === "+" && "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
              line.kind === "-" && "bg-critical/10 text-critical",
              line.kind === " " && "text-text-dim",
            )}
          >
            <span className="opacity-50 mr-2">{line.kind}</span>
            {line.text || " "}
          </div>
        ))}
        {!expanded && hasMore && (
          <button
            onClick={() => setExpanded(true)}
            className="w-full px-3 py-1 text-[10.5px] text-text-muted hover:text-text text-left"
          >
            … 还有 {patch.diff_lines.length - previewLines.length} 行
          </button>
        )}
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Step 4：应用结果
// ──────────────────────────────────────────────────────────────────
function Step4Result({ result }: { result: ApplyBatchResult }) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-3">
        {result.failure_count === 0 ? (
          <CheckCircle2 className="w-5 h-5 text-emerald-500" />
        ) : (
          <AlertTriangle className="w-5 h-5 text-amber-500" />
        )}
        <h4 className="text-[14px] font-semibold">
          {result.dry_run && "[Dry-run] "}
          {result.success_count} 成功
          {result.failure_count > 0 && ` · ${result.failure_count} 失败`}
        </h4>
      </div>

      <div className="text-[11px] text-text-muted mb-3 font-mono">
        Batch ID：{result.batch_id}
      </div>

      <div className="space-y-1.5">
        {result.outcomes.map((o) => (
          <div
            key={o.agent_id}
            className={cn(
              "flex items-start gap-2 px-3 py-2 rounded-md border",
              o.success
                ? "border-emerald-500/30 bg-emerald-500/5"
                : "border-critical/30 bg-critical/5",
            )}
          >
            {o.success ? (
              <CheckCircle2 className="w-3.5 h-3.5 mt-0.5 text-emerald-500 flex-shrink-0" />
            ) : (
              <XCircle className="w-3.5 h-3.5 mt-0.5 text-critical flex-shrink-0" />
            )}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-1.5 text-[12px]">
                <span className="font-medium">{o.agent_name}</span>
                <span className="text-[10.5px] text-text-muted">
                  {platformLabel(o.agent_platform)}
                </span>
              </div>
              <div className="text-[10.5px] font-mono text-text-muted truncate">
                {o.config_path}
              </div>
              <div className="text-[11px] text-text-dim mt-0.5">{o.message}</div>
            </div>
          </div>
        ))}
      </div>

      {result.dry_run && (
        <div className="mt-4 px-3 py-2.5 rounded-md bg-amber-500/10 border border-amber-500/30 text-[11.5px] text-amber-700 dark:text-amber-300">
          以上变更写入了 dry-run 沙箱（~/.clawheart-v2/dry-run/）。检查无误后可在 W8 阶段
          开启实际写入。可在「模型管理 → 应用记录」查看与回滚。
        </div>
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// helpers
// ──────────────────────────────────────────────────────────────────
function platformLabel(p: string): string {
  const map: Record<string, string> = {
    cursor: "Cursor",
    claude: "Claude Code",
    continue: "Continue.dev",
    openclaw: "OpenClaw",
    codex: "Codex CLI",
    gemini: "Gemini CLI",
    windsurf: "Windsurf",
  };
  return map[p] ?? p;
}


function sourceLabel(s: ConfigPatch["source"]): string {
  switch (s.type) {
    case "JsonFile":
      return `${s.path} · ${s.json_path}`;
    case "TomlFile":
      return `${s.path} · ${s.key}`;
    case "EnvVar":
      return `env: ${s.name}`;
    case "VsCodeWorkspace":
      return `vsx: ${s.path}`;
    case "Unknown":
      return "未知配置源";
  }
}

