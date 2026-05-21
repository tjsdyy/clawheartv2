import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";

export interface AdvisoryItem {
  id: string;
  severity: "critical" | "high" | "medium" | "low";
  title: string;
  cvss_score: number | null;
  published: string;
  matched_locally: boolean;
  dismissed: boolean;
}

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

const MOCK: AdvisoryItem[] = [
  { id: "CVE-2026-8305", severity: "high",
    title: "@some/postgres-mcp 同形字符 description 注入",
    cvss_score: 7.8, published: "2026-05-15 10:08:55",
    matched_locally: true, dismissed: false },
  { id: "CVE-2026-8201", severity: "medium",
    title: "Claude Code <=0.7.2 MCP tool description 缓存未失效",
    cvss_score: 5.4, published: "2026-05-12 14:00:00",
    matched_locally: false, dismissed: false },
];

export function useAdvisories() {
  return useQuery({
    queryKey: QK.advisories,
    queryFn: async () =>
      inTauri ? invoke<AdvisoryItem[]>("list_advisories") : MOCK,
    staleTime: 6 * 60 * 60 * 1000, // 6h（与 feed 同步周期对齐）
  });
}

export function useDismissAdvisory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) =>
      inTauri ? invoke<void>("acknowledge_advisory", { id }) : Promise.resolve(),
    onSuccess: () => qc.invalidateQueries({ queryKey: QK.advisories }),
  });
}
