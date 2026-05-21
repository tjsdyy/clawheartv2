import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, Globe, Power, Folder, Bell, Shield, Wrench, ArrowRight, RotateCcw, ShieldAlert, Loader2, FileCheck } from "lucide-react";
import { SecurityRulesPanel } from "./SecurityRulesPanel";
import { useTheme, THEMES, type ThemeName } from "@/hooks/useTheme";
import { useSettings, useUpdateSettings } from "@/hooks/useSettings";
import { useOnboarding } from "@/hooks/useOnboarding";
import { useAccessMode } from "@/hooks/useAccessMode";
import { useApplyRealStatus, useSetApplyRealEnabled } from "@/hooks/useAgentConfig";
import { getTier } from "@/components/access-mode/data";
import { ApplyRealRiskDialog } from "@/components/access-mode/ApplyRealRiskDialog";
import { setLanguage as applyI18nLang, SUPPORTED_LANGUAGES } from "@/lib/i18n";
import { cn } from "@/lib/utils";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";

const SECTIONS = [
  { id: "general", label: "通用", icon: Wrench },
  { id: "appearance", label: "外观", icon: Globe },
  { id: "proxy", label: "代理", icon: Power },
  { id: "security", label: "安全", icon: Shield },
  { id: "security_rules", label: "安全规则", icon: FileCheck },
  { id: "notifications", label: "通知", icon: Bell },
  { id: "advanced", label: "高级", icon: Folder },
];

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export function SettingsTool() {
  const navigate = useNavigate();
  const [section, setSection] = useState("general");
  const { theme, setTheme } = useTheme();
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const { data: accessMode } = useAccessMode();
  const { data: applyReal } = useApplyRealStatus();
  const setApplyReal = useSetApplyRealEnabled();
  const resetOnboarding = useOnboarding((s) => s.setCompleted);

  const currentTier = accessMode?.current_tier ?? "tier1";
  const currentTierMeta = getTier(currentTier);
  const [riskDialogOpen, setRiskDialogOpen] = useState(false);

  async function handleToggleApplyReal(next: boolean) {
    if (!next) {
      // 关闭无需确认
      await setApplyReal.mutateAsync({ enabled: false, acknowledged: true });
      toast.success("已切回 dry-run 沙箱模式");
      return;
    }
    // 开启走二次确认 Dialog
    setRiskDialogOpen(true);
  }

  async function handleConfirmEnableReal() {
    try {
      await setApplyReal.mutateAsync({ enabled: true, acknowledged: true });
      setRiskDialogOpen(false);
      toast.warning("实际写入已启用 · 后续「自动应用」将修改真实文件");
    } catch {
      // toast 已在 hook
    }
  }

  function handleReplayOnboarding() {
    if (
      !confirm(
        "确认重新走监控模式引导？\n应用会回到欢迎页，重新选择监控模式。已有的拦截记录和配置不会被清除。",
      )
    )
      return;
    resetOnboarding(false);
    toast.success("引导已重置 · 下次界面渲染时生效");
  }

  // 本地表单状态 — 从 settings 同步
  const [startOnLogin, setStartOnLogin] = useState(false);
  const [compactMode, setCompactMode] = useState(false);
  const [language, setLanguage] = useState(settings?.language ?? "zh");
  const [proxyPort, setProxyPort] = useState(19111);
  const [failClosedOnPanic, setFailClosedOnPanic] = useState(true);

  useEffect(() => {
    if (settings) {
      setStartOnLogin(settings.start_with_system ?? false);
      setCompactMode(settings.compact_mode ?? false);
      setLanguage(settings.language ?? "zh");
    }
  }, [settings]);

  function saveAll(patch: Partial<typeof settings>) {
    if (!settings) return;
    update.mutate({ ...settings, ...patch } as any);
  }

  function handleLangChange(lang: string) {
    setLanguage(lang);
    applyI18nLang(lang);
    saveAll({ language: lang });
  }

  function copyCertPath() {
    navigator.clipboard.writeText("~/.clawheart-v2/ca/clawheart-ca.pem");
    toast.success("已复制路径");
  }

  function copyDataDir() {
    navigator.clipboard.writeText("~/.clawheart-v2");
    toast.success("已复制路径");
  }


  async function handleReset() {
    if (!confirm("确认清除所有本地数据？此操作不可逆。")) return;
    localStorage.clear();
    if (inTauri) {
      try {
        await invoke("logout");
      } catch {}
    }
    toast.success("已重置 · 重启 ClawHeart 生效");
  }

  return (
    <div className="grid h-full" style={{ gridTemplateColumns: "200px 1fr" }}>
      <aside className="border-r border-border-soft p-3 overflow-auto">
        {SECTIONS.map((s) => {
          const Icon = s.icon;
          return (
            <button
              key={s.id}
              onClick={() => setSection(s.id)}
              className={cn(
                "w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-[13px] text-left transition-colors mb-0.5",
                section === s.id ? "bg-bg-elev2 text-text font-medium" : "text-text-dim hover:text-text hover:bg-bg-elev2",
              )}
            >
              <Icon className="w-4 h-4" />
              {s.label}
            </button>
          );
        })}
      </aside>

      {section === "security_rules" ? (
        <div className="overflow-hidden">
          <SecurityRulesPanel />
        </div>
      ) : (
      <div className="overflow-auto p-8" style={{ maxWidth: 680 }}>
        {section === "general" && (
          <SettingsSection title="通用">
            <Row label="开机启动" desc="登录系统后自动启动 ClawHeart">
              <Toggle checked={startOnLogin} onChange={(v) => { setStartOnLogin(v); saveAll({ start_with_system: v }); }} />
            </Row>
            <Row label="紧凑模式" desc="窗口宽度 < 600px 时自动启用 mini 模式">
              <Toggle checked={compactMode} onChange={(v) => { setCompactMode(v); saveAll({ compact_mode: v }); }} />
            </Row>
            <Row label="界面语言" desc="切换后立即生效">
              <select
                value={language}
                onChange={(e) => handleLangChange(e.target.value)}
                className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[13px] outline-none focus:border-accent"
              >
                {SUPPORTED_LANGUAGES.map((l) => (
                  <option key={l.code} value={l.code}>{l.label}</option>
                ))}
              </select>
            </Row>
          </SettingsSection>
        )}

        {section === "appearance" && (
          <SettingsSection title="外观主题">
            <div className="grid grid-cols-1 gap-2">
              {THEMES.map((t) => (
                <button
                  key={t.id}
                  onClick={() => setTheme(t.id as ThemeName)}
                  className={cn(
                    "flex items-center gap-3 p-3 rounded-lg border transition-colors text-left",
                    theme === t.id ? "border-accent bg-accent/5" : "border-border hover:border-text-muted",
                  )}
                >
                  <span
                    className="w-10 h-10 rounded-lg flex-shrink-0 border border-border"
                    style={{ background: t.swatch }}
                  />
                  <div className="flex-1">
                    <div className="text-[14px] font-medium">{t.label}</div>
                    <div className="text-[11.5px] text-text-muted">{t.hint}</div>
                  </div>
                  {theme === t.id && <Check className="w-4 h-4 text-accent flex-shrink-0" />}
                </button>
              ))}
            </div>
          </SettingsSection>
        )}

        {section === "proxy" && (
          <SettingsSection title="代理服务">
            <Row
              label="当前监控模式"
              desc={`${currentTierMeta.name} · ${currentTierMeta.subtitle}`}
            >
              <button
                onClick={() => navigate("/tools/access_mode")}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-bg-elev2 border border-border hover:border-text-muted text-[12.5px]"
              >
                查看与切换
                <ArrowRight className="w-3 h-3" />
              </button>
            </Row>
            <Row
              label="重新走监控模式引导"
              desc="回到欢迎页，重新选择监控模式（已有数据不会清除）"
            >
              <button
                onClick={handleReplayOnboarding}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12.5px] border border-border hover:border-text-muted"
              >
                <RotateCcw className="w-3 h-3" />
                重新引导
              </button>
            </Row>
            <Row label="代理端口" desc="本机 MITM 代理监听端口（仅 127.0.0.1）">
              <input
                type="number"
                value={proxyPort}
                onChange={(e) => setProxyPort(Number(e.target.value))}
                className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[13px] font-mono w-32 outline-none focus:border-accent"
              />
            </Row>
            <Row label="CA 证书路径" desc="点击复制到剪贴板">
              <button onClick={copyCertPath} className="btn-ghost text-[12px] font-mono">
                ~/.clawheart-v2/ca/clawheart-ca.pem
              </button>
            </Row>
            <Row label="启动时检查 CA 信任" desc="未信任时自动提醒去监控模式工具页安装">
              <Toggle checked={true} onChange={() => toast.info("启动检测已默认启用")} />
            </Row>
          </SettingsSection>
        )}

        {section === "security" && (
          <SettingsSection title="安全策略">
            <Row
              label="实际写入 Agent 配置文件"
              desc={
                applyReal?.enabled
                  ? "⚠ 已启用：「自动应用到 Agent」会修改真实配置文件；变更受 snapshot 保护，可回滚"
                  : "默认关闭：「自动应用到 Agent」仅写入 ~/.clawheart-v2/dry-run/ 沙箱目录"
              }
            >
              <div className="flex items-center gap-2">
                {applyReal?.enabled && (
                  <span className="inline-flex items-center gap-1 text-[10px] font-medium px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300">
                    <ShieldAlert className="w-2.5 h-2.5" />
                    实际写入
                  </span>
                )}
                <Toggle
                  checked={applyReal?.enabled ?? false}
                  onChange={handleToggleApplyReal}
                />
              </div>
            </Row>
            <Row label="panic 时失败关闭" desc="安全检查崩溃 → 阻止请求（强烈建议保持开启）">
              <Toggle checked={failClosedOnPanic} onChange={setFailClosedOnPanic} />
            </Row>
            <Row label="MCP 工具基线" desc="会话开始时冻结 MCP 工具描述哈希（W11 启用）">
              <Toggle checked={true} onChange={() => toast.info("MCP 工具基线在 W11 启用")} />
            </Row>
          </SettingsSection>
        )}

        {section === "notifications" && (
          <SettingsSection title="通知设置">
            <Row label="关键拦截系统通知" desc="critical 级事件触发 macOS/Windows 系统通知（需 notifications feature）">
              <Toggle checked={true} onChange={() => toast.info("通知接入 W14")} />
            </Row>
            <Row label="预算 80% 告警" desc="到达阈值时桌面通知">
              <Toggle checked={true} onChange={() => toast.info("通知接入 W14")} />
            </Row>
            <Row label="公告匹配告警" desc="本机技能/Agent 命中新公告时通知">
              <Toggle checked={true} onChange={() => toast.info("通知接入 W14")} />
            </Row>
          </SettingsSection>
        )}

        {riskDialogOpen && (
          <ApplyRealRiskDialog
            onClose={() => setRiskDialogOpen(false)}
            onConfirm={handleConfirmEnableReal}
            busy={setApplyReal.isPending}
          />
        )}

        {section === "advanced" && (
          <SettingsSection title="高级">
            <Row label="数据目录" desc="ClawHeart 配置与数据库存储位置">
              <button onClick={copyDataDir} className="btn-ghost text-[12px] font-mono">
                ~/.clawheart-v2
              </button>
            </Row>
            {/* 导出诊断包 / 专家模式 — 待真实接通后开放 */}
            <Row label="清除所有本地数据" desc="重置到初始状态（不可逆）">
              <button
                onClick={handleReset}
                className="text-[12px] px-3 py-1.5 rounded-md border border-critical/30 text-critical hover:bg-critical/5"
              >
                重置…
              </button>
            </Row>
          </SettingsSection>
        )}
      </div>
      )}
    </div>
  );
}

function SettingsSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="text-[18px] font-semibold tracking-tight mb-4">{title}</h2>
      <div className="space-y-1">{children}</div>
    </div>
  );
}

function Row({
  label,
  desc,
  children,
}: {
  label: string;
  desc?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-6 py-3.5 border-b border-border-soft">
      <div className="flex-1 min-w-0">
        <div className="text-[13.5px] font-medium">{label}</div>
        {desc && <div className="text-[12px] text-text-dim mt-0.5 leading-snug">{desc}</div>}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={cn(
        "w-10 h-6 rounded-full relative transition-colors",
        checked ? "bg-accent" : "bg-bg-elev2 border border-border",
      )}
    >
      <span
        className={cn(
          "absolute top-0.5 w-5 h-5 bg-white rounded-full shadow-sm transition-transform",
          checked ? "translate-x-[18px]" : "translate-x-0.5",
        )}
      />
    </button>
  );
}
