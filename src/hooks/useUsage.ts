import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface UsageSummary {
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cost_usd: number;
  request_count: number;
  blocked_count: number;
}
export interface UsageDay {
  date: string;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
}
export interface UsageProviderRow {
  provider: string;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
  request_count: number;
}
export interface UsageModelRow {
  provider: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
  request_count: number;
}

const EMPTY_SUMMARY: UsageSummary = {
  input_tokens: 0,
  output_tokens: 0,
  cache_read_tokens: 0,
  cost_usd: 0,
  request_count: 0,
  blocked_count: 0,
};

export function useUsageSummary() {
  return useQuery({
    queryKey: ["usage", "summary"],
    queryFn: async () =>
      inTauri ? invoke<UsageSummary>("get_usage_summary") : EMPTY_SUMMARY,
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
}

export function useUsageTrends(days = 14) {
  return useQuery({
    queryKey: ["usage", "trends", days],
    queryFn: async () =>
      inTauri ? invoke<UsageDay[]>("get_usage_trends", { days }) : ([] as UsageDay[]),
    staleTime: 60_000,
  });
}

export function useUsageByProvider(days = 30) {
  return useQuery({
    queryKey: ["usage", "by_provider", days],
    queryFn: async () =>
      inTauri
        ? invoke<UsageProviderRow[]>("get_usage_by_provider", { days })
        : ([] as UsageProviderRow[]),
    staleTime: 60_000,
  });
}

export function useUsageByModel(days = 30) {
  return useQuery({
    queryKey: ["usage", "by_model", days],
    queryFn: async () =>
      inTauri
        ? invoke<UsageModelRow[]>("get_usage_by_model", { days })
        : ([] as UsageModelRow[]),
    staleTime: 60_000,
  });
}
