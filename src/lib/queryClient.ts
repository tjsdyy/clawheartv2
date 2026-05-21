import { QueryClient } from "@tanstack/react-query";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000,
      refetchOnWindowFocus: true,
      retry: 1,
      retryDelay: (attempt) => Math.min(1000 * 2 ** attempt, 8000),
    },
    mutations: {
      retry: 0,
    },
  },
});

export const QK = {
  status: ["status"] as const,
  settings: ["settings"] as const,
  events: ["events"] as const,
  events_recent: ["events", "recent"] as const,
  tools: ["tools"] as const,
  agents: ["agents"] as const,
  skills: (tab: string) => ["skills", tab] as const,
  budget: ["budget"] as const,
  advisories: ["advisories"] as const,
  request_logs: (limit: number) => ["request_logs", limit] as const,
  scan_history: ["scan", "history"] as const,
  scan_items: ["scan", "items"] as const,
  access_mode: ["access_mode"] as const,
  ca_status: ["ca_status"] as const,
};
