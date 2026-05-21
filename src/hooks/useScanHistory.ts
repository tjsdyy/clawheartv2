import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface ScanHistoryItem {
  id: number;
  started_at: string;
  completed_at: string | null;
  total: number;
  passed: number;
  failed: number;
  warned: number;
  skipped: number;
}

export function useScanHistory() {
  return useQuery({
    queryKey: QK.scan_history,
    queryFn: async () =>
      inTauri ? invoke<ScanHistoryItem[]>("list_scan_history") : ([] as ScanHistoryItem[]),
    staleTime: 5_000,
  });
}

export interface CheckResultItem {
  id: string;
  category: string;
  outcome: "pass" | "fail" | "warn" | "skipped";
  description: string;
  detail: string | null;
  remediation: string | null;
}

export interface ScanRunDetailRaw {
  id: number;
  started_at: string;
  completed_at: string | null;
  items: string[];
  results_json: string;
  total: number;
  passed: number;
  failed: number;
  warned: number;
  skipped: number;
}

export interface ScanRunDetail extends Omit<ScanRunDetailRaw, "results_json"> {
  results: CheckResultItem[];
}

export function useScanRun(id: number | null) {
  return useQuery({
    queryKey: ["scan_run", id],
    enabled: id !== null,
    queryFn: async (): Promise<ScanRunDetail> => {
      if (!inTauri) {
        return {
          id: id ?? 0, started_at: "2026-05-19 11:00:00", completed_at: "2026-05-19 11:00:01",
          items: ["FilePermission", "AgentBehavior"], total: 18, passed: 9, failed: 2, warned: 1, skipped: 6,
          results: [
            { id: "FP-001", category: "FilePermission", outcome: "pass",
              description: "~/.claude 目录权限不宽松", detail: "755", remediation: null },
            { id: "FP-003", category: "FilePermission", outcome: "fail",
              description: ".env 权限 600", detail: "权限 644, 期望 600", remediation: "chmod 600 .env" },
            { id: "AB-002", category: "AgentBehavior", outcome: "skipped",
              description: "Codex 未启用 dangerous-skip-permissions", detail: "未实现 · 待 W21 接入", remediation: null },
          ],
        };
      }
      const raw = await invoke<ScanRunDetailRaw>("get_scan_run", { id });
      let results: CheckResultItem[] = [];
      try {
        results = JSON.parse(raw.results_json);
      } catch {
        results = [];
      }
      const { results_json: _omit, ...rest } = raw;
      void _omit;
      return { ...rest, results };
    },
    staleTime: 60_000,
  });
}
