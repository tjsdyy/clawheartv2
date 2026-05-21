import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface BudgetRuleItem {
  id: number;
  provider: string;
  model: string | null;
  period: "daily" | "monthly";
  limit_usd: number;
  used_usd: number;
  enabled: boolean;
}

const MOCK: BudgetRuleItem[] = [
  { id: 1, provider: "global", model: null, period: "daily", limit_usd: 10, used_usd: 2.14, enabled: true },
  { id: 2, provider: "anthropic", model: "claude-opus-4-7", period: "daily", limit_usd: 5, used_usd: 1.83, enabled: true },
  { id: 3, provider: "openai", model: "gpt-5.4-thinking", period: "daily", limit_usd: 3, used_usd: 0.31, enabled: true },
  { id: 4, provider: "anthropic", model: null, period: "monthly", limit_usd: 150, used_usd: 47.21, enabled: true },
  { id: 5, provider: "google", model: null, period: "monthly", limit_usd: 30, used_usd: 0, enabled: false },
];

export function useBudgetRules() {
  return useQuery({
    queryKey: QK.budget,
    queryFn: async () =>
      inTauri ? invoke<BudgetRuleItem[]>("list_budget_rules") : MOCK,
    staleTime: 60_000,
  });
}

export interface TokenUsageDay {
  date: string;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
}

const USAGE_MOCK: TokenUsageDay[] = [
  { date: "2026-05-13", input_tokens: 184_320, output_tokens: 22_180, cost_usd: 1.42 },
  { date: "2026-05-14", input_tokens: 211_064, output_tokens: 25_902, cost_usd: 1.71 },
  { date: "2026-05-15", input_tokens: 95_400,  output_tokens: 11_220, cost_usd: 0.78 },
  { date: "2026-05-16", input_tokens: 262_180, output_tokens: 31_408, cost_usd: 2.04 },
  { date: "2026-05-17", input_tokens: 308_590, output_tokens: 38_770, cost_usd: 2.41 },
  { date: "2026-05-18", input_tokens: 244_220, output_tokens: 29_180, cost_usd: 1.93 },
  { date: "2026-05-19", input_tokens: 153_004, output_tokens: 18_602, cost_usd: 1.18 },
];

export function useTokenUsage(days = 7) {
  return useQuery({
    queryKey: ["token_usage", days],
    queryFn: async () =>
      inTauri ? invoke<TokenUsageDay[]>("get_token_usage", { days }) : USAGE_MOCK,
    staleTime: 30_000,
  });
}
