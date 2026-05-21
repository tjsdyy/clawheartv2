import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface RequestLogItem {
  id: number;
  timestamp: string;
  agent_id: string | null;
  format: string;
  provider: string | null;
  model: string | null;
  endpoint: string;
  method: string;
  status_code: number;
  blocked: boolean;
  bytes_in: number;
  bytes_out: number;
  latency_ms: number;
}

function generateMock(limit: number): RequestLogItem[] {
  const agents = ["Claude Code", "Codex CLI", "Cursor", "Gemini CLI"];
  const endpoints = ["/v1/messages", "/v1/responses", "/v1/chat/completions", ":generateContent"];
  const providers = ["anthropic", "openai", "openai", "google"];
  const models = ["claude-opus-4-7", "gpt-5.4-thinking", "gpt-5.4", "gemini-2.5-pro"];
  const statuses = [200, 200, 200, 200, 200, 429, 0]; // 0 = blocked

  return Array.from({ length: limit }, (_, i) => {
    const idx = i % 4;
    const status = statuses[i % 7];
    return {
      id: 1000 - i,
      timestamp: `${(15 - Math.floor(i / 60)) % 24}:${(60 - (i % 60)) % 60}:${(45 - i) % 60}`.slice(0, 8),
      agent_id: agents[idx],
      format: providers[idx] === "anthropic" ? "claude" : providers[idx] === "google" ? "gemini" : "openai",
      provider: providers[idx],
      model: models[idx],
      endpoint: endpoints[idx],
      method: "POST",
      status_code: status === 0 ? 403 : status,
      blocked: status === 0,
      bytes_in: 1200 + (i * 47) % 800,
      bytes_out: 4800 + (i * 113) % 12000,
      latency_ms: 320 + (i * 41) % 1600,
    };
  });
}

export function useRequestLogs(limit = 30) {
  return useQuery({
    queryKey: QK.request_logs(limit),
    queryFn: async () =>
      inTauri ? invoke<RequestLogItem[]>("list_request_logs", { limit }) : generateMock(limit),
    staleTime: 15_000,
    refetchInterval: 30_000,
  });
}
