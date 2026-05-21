import { useEffect } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { AppShell } from "./components/AppShell";
import { ToolsGrid } from "./components/grid/ToolsGrid";
import { ToolLayout } from "./components/tools/ToolLayout";
import { MonitorTool } from "./components/tools/MonitorTool";
import { ScanTool } from "./components/tools/ScanTool";
import { SkillsBackupTool } from "./components/tools/SkillsBackupTool";
import { AdvisoryTool } from "./components/tools/AdvisoryTool";
import { LogsTool } from "./components/tools/LogsTool";
import { BudgetTool } from "./components/tools/BudgetTool";
import { AuditTool } from "./components/tools/AuditTool";
import { OpenClawTool } from "./components/tools/OpenClawTool";
import { AgentsTool } from "./components/tools/AgentsTool";
import { SettingsTool } from "./components/tools/SettingsTool";
import { SoonTool } from "./components/tools/PlaceholderTool";
import { Onboarding } from "./components/tools/Onboarding";
import { AccessModeTool } from "./components/access-mode/AccessModeTool";
import { ProvidersTool } from "./components/providers/ProvidersTool";
import { UsageTool } from "./components/tools/UsageTool";
import { CommandPalette } from "./components/overlays/CommandPalette";
import { TrayPopup } from "./components/overlays/TrayPopup";
import { ThemePicker } from "./components/overlays/ThemePicker";
import { useTheme } from "./hooks/useTheme";
import { useOverlays } from "./hooks/useOverlays";
import { useOnboarding } from "./hooks/useOnboarding";

function App() {
  const { theme } = useTheme();
  const { toggleCmdk, closeAll } = useOverlays();
  const { completed } = useOnboarding();

  // Apply theme on mount (in case storage hasn't rehydrated yet)
  useEffect(() => {
    document.documentElement.classList.add(`theme-${theme}`);
  }, [theme]);

  // Global hotkeys
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        toggleCmdk();
      }
      if (e.key === "Escape") closeAll();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggleCmdk, closeAll]);

  if (!completed) {
    return (
      <>
        <Onboarding />
        <CommandPalette />
        <ThemePicker />
      </>
    );
  }

  return (
    <>
      <AppShell>
        <Routes>
          <Route path="/" element={<ToolsGrid />} />
          <Route path="/tools/access_mode" element={<ToolLayout title="监控模式"><AccessModeTool /></ToolLayout>} />
          <Route path="/tools/monitor" element={<ToolLayout title="实时监控" tabs={["实时流", "拦截记录", "Token 用量", "预算"]}><MonitorTool /></ToolLayout>} />
          <Route path="/tools/scan" element={<ToolLayout title="安全扫描"><ScanTool /></ToolLayout>} />
          <Route path="/tools/skills" element={<ToolLayout title="技能管理" tabs={["本机技能", "扫描报告", "备份历史"]}><SkillsBackupTool /></ToolLayout>} />
          <Route path="/tools/advisory" element={<ToolLayout title="安全公告"><AdvisoryTool /></ToolLayout>} />
          <Route path="/tools/logs" element={<ToolLayout title="请求日志"><LogsTool /></ToolLayout>} />
          <Route path="/tools/budget" element={<ToolLayout title="预算"><BudgetTool /></ToolLayout>} />
          <Route path="/tools/audit" element={<ToolLayout title="审计报告"><AuditTool /></ToolLayout>} />
          <Route path="/tools/openclaw" element={<ToolLayout title="OpenClaw 集成"><OpenClawTool /></ToolLayout>} />
          <Route path="/tools/agents" element={<ToolLayout title="Agent 发现"><AgentsTool /></ToolLayout>} />
          <Route path="/tools/settings" element={<ToolLayout title="设置"><SettingsTool /></ToolLayout>} />
          <Route path="/tools/token_verify" element={<ToolLayout title="Token 验真"><SoonTool toolId="token_verify" version="v2.1" /></ToolLayout>} />
          <Route path="/tools/providers" element={<ToolLayout title="模型管理"><ProvidersTool /></ToolLayout>} />
          <Route path="/tools/usage" element={<ToolLayout title="用量统计"><UsageTool /></ToolLayout>} />
          <Route path="/tools/policy" element={<ToolLayout title="企业策略"><SoonTool toolId="policy" version="v2.2" /></ToolLayout>} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </AppShell>
      <CommandPalette />
      <TrayPopup />
      <ThemePicker />
    </>
  );
}

export default App;
