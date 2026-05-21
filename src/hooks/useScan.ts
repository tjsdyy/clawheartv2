import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface ScanItemGroup {
  category: string;
  label: string;
  count: number;
}

export interface CheckResult {
  id: string;
  category: string;
  outcome: "pass" | "fail" | "warn" | "skipped";
  description: string;
  detail: string | null;
  remediation: string | null;
}

export interface ScanRunResult {
  run_id: number;
  total: number;
  passed: number;
  failed: number;
  warned: number;
  skipped: number;
  results: CheckResult[];
}

const MOCK_ITEMS: ScanItemGroup[] = [
  { category: "FilePermission",   label: "文件权限",     count: 10 },
  { category: "McpConfig",        label: "MCP 配置",     count: 5 },
  { category: "CredentialLeak",   label: "凭据泄露",     count: 8 },
  { category: "AgentBehavior",    label: "Agent 行为",   count: 8 },
  { category: "SkillSupplyChain", label: "技能供应链",   count: 12 },
  { category: "SandboxDocker",    label: "沙箱与 Docker", count: 11 },
  { category: "NetworkExposure",  label: "网络暴露",     count: 9 },
  { category: "WindowsSpecific",  label: "Windows 专属", count: 2 },
];

export function useScanItems() {
  return useQuery({
    queryKey: QK.scan_items,
    queryFn: async () =>
      inTauri ? invoke<ScanItemGroup[]>("get_scan_items") : MOCK_ITEMS,
    staleTime: Infinity,
  });
}

export function useStartScanRun() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (items: string[]) => {
      if (!inTauri) {
        return {
          run_id: 1, total: 73, passed: 67, failed: 2, warned: 3, skipped: 1,
          results: [],
        } as ScanRunResult;
      }
      return invoke<ScanRunResult>("start_scan_run", { items });
    },
    onSuccess: (run) => {
      qc.invalidateQueries({ queryKey: QK.scan_history });
      const summary = `${run.failed} 严重 · ${run.warned} 警告 · ${run.passed} 通过`;
      if (run.failed > 0) toast.error(`扫描完成：${summary}`);
      else toast.success(`扫描完成：${summary}`);
    },
    onError: (err) => toast.error(`扫描失败：${err}`),
  });
}
