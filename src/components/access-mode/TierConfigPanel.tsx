import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Copy, ShieldCheck, ShieldOff, Cpu, Loader2, Check, AlertTriangle, Radio, ArrowRight } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import {
  useProtocolAdapters,
  useToggleProtocolAdapter,
  useUpdateProxyPort,
  type AccessModeInfo,
  type AccessTier,
  type CaStatus,
  type ProtocolAdapter,
} from "@/hooks/useAccessMode";

interface Props {
  tier: AccessTier;
  isCurrent: boolean;
  mode: AccessModeInfo;
  caStatus?: CaStatus;
  caBusy?: boolean;
  onInstallCa: () => void;
  onUninstallCa: () => void;
  onOpenSandbox: () => void;
}

export function TierConfigPanel({
  tier,
  isCurrent,
  mode,
  caStatus,
  caBusy,
  onInstallCa,
  onUninstallCa,
  onOpenSandbox,
}: Props) {
  return (
    <div
      className={cn(
        "rounded-md border px-3 py-2.5 flex flex-col min-h-[230px]",
        isCurrent
          ? "bg-accent/5 border-accent/30"
          : "bg-bg/40 border-border-soft opacity-90",
      )}
    >
      <div className="flex items-center justify-between mb-1.5">
        <div className="text-[10.5px] text-text-muted uppercase tracking-wider">
          配置参数
        </div>
        {!isCurrent && (
          <span className="text-[10px] text-text-muted">
            非当前模式 · 切换后生效
          </span>
        )}
      </div>

      <div className="flex-1 space-y-2">
        {tier === "tier1" && (
          <Tier1Panel
            mode={mode}
            editable={isCurrent}
          />
        )}

        {tier === "tier2" && (
          <Tier2Panel
            mode={mode}
            editable={isCurrent}
            caStatus={caStatus}
            caBusy={caBusy}
            onInstallCa={onInstallCa}
            onUninstallCa={onUninstallCa}
          />
        )}

        {tier === "tier3" && (
          <Tier3Panel onOpenSandbox={onOpenSandbox} />
        )}
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// Tier 1：端点映射
// ──────────────────────────────────────────────────────────────────
function Tier1Panel({
  mode,
  editable,
}: {
  mode: AccessModeInfo;
  editable: boolean;
}) {
  const navigate = useNavigate();
  return (
    <>
      <ConfigRow
        label="反向代理端点"
        value={mode.fetch_url_template}
        mono
        onCopy={() => copyValue(mode.fetch_url_template, "反向代理端点已复制")}
      />
      <PortRow
        label="监听端口"
        tier="tier1"
        currentPort={mode.reverse_proxy_port}
        editable={editable}
      />
      <ProtocolAdaptersSection editable={editable} />
      <div className="pt-2 mt-1 border-t border-border-soft flex items-center justify-between gap-2">
        <div className="flex items-center gap-1.5 text-[11px] text-text-muted leading-snug">
          <Radio className="w-3 h-3 flex-shrink-0" />
          <span>在「Agent 发现」选择模型渠道 · 一键应用</span>
        </div>
        <button
          onClick={() => navigate("/tools/agents")}
          className="flex items-center gap-1 px-2.5 py-1 rounded text-[11.5px] font-medium bg-bg-elev2 border border-border hover:border-text-muted"
        >
          Agent 管理
          <ArrowRight className="w-3 h-3" />
        </button>
      </div>
    </>
  );
}

// ──────────────────────────────────────────────────────────────────
// Tier 2：系统代理
// ──────────────────────────────────────────────────────────────────
function Tier2Panel({
  mode,
  editable,
  caStatus,
  caBusy,
  onInstallCa,
  onUninstallCa,
}: {
  mode: AccessModeInfo;
  editable: boolean;
  caStatus?: CaStatus;
  caBusy?: boolean;
  onInstallCa: () => void;
  onUninstallCa: () => void;
}) {
  return (
    <>
      <ConfigRow
        label="自签 CA 证书"
        value={
          caStatus?.installed
            ? `已信任 · ${caStatus.fingerprint ?? "—"}`
            : "未安装"
        }
        mono
      />
      <div className="flex items-center justify-between gap-2 py-0.5">
        <span className="text-[11.5px] text-text-dim">证书管理</span>
        <div className="flex items-center gap-1.5">
          {caStatus?.installed ? (
            <button
              onClick={onUninstallCa}
              disabled={caBusy}
              className="flex items-center gap-1 text-[11.5px] px-2.5 py-1 rounded border border-critical/30 text-critical hover:bg-critical/5 disabled:opacity-40"
            >
              <ShieldOff className="w-3 h-3" />
              卸载
            </button>
          ) : (
            <button
              onClick={onInstallCa}
              disabled={caBusy}
              className="flex items-center gap-1 text-[11.5px] px-2.5 py-1 rounded bg-accent text-white hover:bg-accent/90 disabled:opacity-50"
            >
              {caBusy ? (
                <Loader2 className="w-3 h-3 animate-spin" />
              ) : (
                <ShieldCheck className="w-3 h-3" />
              )}
              安装
            </button>
          )}
        </div>
      </div>
      <PortRow
        label="正向代理监听"
        tier="tier2"
        currentPort={mode.forward_proxy_port}
        editable={editable}
      />
      <ConfigRow
        label="系统代理"
        value={mode.system_proxy_active ? "已启用" : "未启用"}
      />
    </>
  );
}

// ──────────────────────────────────────────────────────────────────
// Tier 3：沙箱隔离
// ──────────────────────────────────────────────────────────────────
function Tier3Panel({
  onOpenSandbox,
}: {
  onOpenSandbox: () => void;
}) {
  return (
    <>
      <ConfigRow label="平台" value={getPlatformLabel()} mono />
      <ConfigRow
        label="启动方式"
        value="clawheart sandbox -- <command>"
        mono
        small
      />
      <div className="flex items-center justify-between gap-2 py-0.5">
        <span className="text-[11.5px] text-text-dim">命令生成器</span>
        <button
          onClick={onOpenSandbox}
          className="flex items-center gap-1 text-[11.5px] px-2.5 py-1 rounded border border-border hover:border-text-muted bg-bg-elev2"
        >
          <Cpu className="w-3 h-3" />
          打开
        </button>
      </div>
      <ConfigRow label="生效状态" value="W20 集成 · 当前为预览" small />
    </>
  );
}

// ──────────────────────────────────────────────────────────────────
// Port Row（可编辑 + 实时校验 + 保存）
// ──────────────────────────────────────────────────────────────────
function PortRow({
  label,
  tier,
  currentPort,
  editable,
}: {
  label: string;
  tier: AccessTier;
  currentPort: number;
  editable: boolean;
}) {
  const [value, setValue] = useState(String(currentPort));
  const [dirty, setDirty] = useState(false);
  const updatePort = useUpdateProxyPort();

  useEffect(() => {
    if (!dirty) setValue(String(currentPort));
  }, [currentPort, dirty]);

  const parsed = Number.parseInt(value, 10);
  const valid = Number.isInteger(parsed) && parsed >= 1024 && parsed <= 65535;
  const helper = !valid
    ? "端口须为 1024-65535 之间的整数"
    : parsed === currentPort
      ? ""
      : `将从 ${currentPort} 修改为 ${parsed}`;

  async function handleSave() {
    if (!valid || parsed === currentPort) return;
    try {
      await updatePort.mutateAsync({ tier, port: parsed });
      setDirty(false);
      toast.success(`${label}已更新为 ${parsed}`);
    } catch {
      // toast 已在 hook 内
    }
  }

  function handleReset() {
    setValue(String(currentPort));
    setDirty(false);
  }

  return (
    <div className="py-0.5">
      <div className="flex items-center justify-between gap-2">
        <span className="text-[11.5px] text-text-dim flex-shrink-0">{label}</span>
        <div className="flex items-center gap-1.5">
          <span className="text-[11.5px] font-mono text-text-muted">127.0.0.1:</span>
          <input
            type="text"
            inputMode="numeric"
            value={value}
            disabled={!editable}
            onChange={(e) => {
              setValue(e.target.value.replace(/[^\d]/g, ""));
              setDirty(true);
            }}
            className={cn(
              "w-20 text-right font-mono text-[11.5px] bg-bg-elev2 border rounded px-2 py-0.5 outline-none",
              valid
                ? "border-border focus:border-accent"
                : "border-amber-500/50 focus:border-amber-500",
              !editable && "opacity-50 cursor-not-allowed",
            )}
          />
          {dirty && valid && parsed !== currentPort && editable && (
            <>
              <button
                onClick={handleSave}
                disabled={updatePort.isPending}
                className="flex items-center gap-0.5 text-[10.5px] px-2 py-0.5 rounded bg-accent text-white hover:bg-accent/90 disabled:opacity-50"
                title="保存"
              >
                {updatePort.isPending ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <Check className="w-3 h-3" />
                )}
                保存
              </button>
              <button
                onClick={handleReset}
                className="text-[10.5px] px-1.5 py-0.5 rounded text-text-muted hover:text-text"
                title="撤销"
              >
                ✕
              </button>
            </>
          )}
        </div>
      </div>
      {helper && (
        <div
          className={cn(
            "mt-0.5 flex items-center gap-1 text-[10.5px] leading-snug",
            valid ? "text-text-muted" : "text-amber-500",
          )}
        >
          {!valid && <AlertTriangle className="w-2.5 h-2.5" />}
          <span>{helper}</span>
        </div>
      )}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// 协议适配器（仅 tier1 显示）
// ──────────────────────────────────────────────────────────────────
function ProtocolAdaptersSection({ editable }: { editable: boolean }) {
  const { data: adapters = [] } = useProtocolAdapters();
  const toggle = useToggleProtocolAdapter();
  const enabledCount = adapters.filter((a) => a.enabled).length;

  return (
    <div className="pt-1.5 mt-1 border-t border-border-soft">
      <div className="flex items-center justify-between mb-1.5">
        <span className="text-[11.5px] text-text-dim">协议兼容路径</span>
        <span className="text-[10.5px] text-text-muted">
          {enabledCount} / {adapters.length} 启用
        </span>
      </div>
      <div className="space-y-0.5">
        {adapters.map((a) => (
          <AdapterRow
            key={a.id}
            adapter={a}
            editable={editable}
            busy={toggle.isPending}
            onToggle={(enabled) =>
              toggle.mutate({ id: a.id, enabled })
            }
          />
        ))}
      </div>
      {editable && enabledCount === 0 && (
        <div className="mt-1 flex items-center gap-1 text-[10.5px] text-amber-500">
          <AlertTriangle className="w-2.5 h-2.5" />
          <span>所有协议已关闭 · 反向代理将拒绝全部请求</span>
        </div>
      )}
    </div>
  );
}

function AdapterRow({
  adapter,
  editable,
  busy,
  onToggle,
}: {
  adapter: ProtocolAdapter;
  editable: boolean;
  busy: boolean;
  onToggle: (enabled: boolean) => void;
}) {
  return (
    <label
      className={cn(
        "flex items-center gap-2 py-0.5 cursor-pointer",
        !editable && "cursor-not-allowed opacity-70",
      )}
    >
      <input
        type="checkbox"
        checked={adapter.enabled}
        disabled={!editable || busy}
        onChange={(e) => onToggle(e.target.checked)}
        className="w-3 h-3 accent-accent"
      />
      <span className="text-[11px] text-text flex-1 min-w-0 truncate">
        {adapter.label}
      </span>
      <code className="text-[10px] font-mono text-text-muted truncate max-w-[160px]">
        {adapter.path}
      </code>
    </label>
  );
}

// ──────────────────────────────────────────────────────────────────
// 共用子组件
// ──────────────────────────────────────────────────────────────────
function ConfigRow({
  label,
  value,
  mono,
  small,
  onCopy,
}: {
  label: string;
  value: string;
  mono?: boolean;
  small?: boolean;
  onCopy?: () => void;
}) {
  return (
    <div className="flex items-center justify-between gap-2 py-0.5 min-w-0">
      <span className="text-[11.5px] text-text-dim flex-shrink-0">{label}</span>
      <div className="flex items-center gap-1 min-w-0">
        <span
          className={cn(
            "truncate text-text",
            mono ? "font-mono" : "",
            small ? "text-[10.5px]" : "text-[11.5px]",
          )}
        >
          {value}
        </span>
        {onCopy && (
          <button
            onClick={onCopy}
            className="text-text-muted hover:text-text flex-shrink-0"
            title="复制"
          >
            <Copy className="w-3 h-3" />
          </button>
        )}
      </div>
    </div>
  );
}

function copyValue(v: string, hint = "已复制") {
  navigator.clipboard.writeText(v);
  toast.success(hint);
}

function getPlatformLabel(): string {
  const ua = navigator.userAgent;
  if (/Mac/.test(ua)) return "macOS · sandbox-exec";
  if (/Win/.test(ua)) return "Windows · AppContainer (v2.1)";
  if (/Linux/.test(ua)) return "Linux · Landlock + seccomp";
  return "未知平台";
}
