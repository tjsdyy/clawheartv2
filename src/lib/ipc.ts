// Tauri IPC 封装。当不在 Tauri 环境（如纯 vite dev）时，降级到「全空」状态而非假数据。
import { invoke } from "@tauri-apps/api/core";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export interface StatusInfo {
  version: string;
  protected: boolean;
  agents: number;
  mcp_servers: number;
  today_requests: number;
  today_blocks: number;
  today_cost_usd: number;
  budget_limit_usd: number;
  proxy_port: number;
  ca_trusted: boolean;
  kill_switch: boolean;
  uptime_sec: number;
  bytes_in: number;
  bytes_out: number;
  last_sync_unix: number;
}

export interface InterceptEvent {
  id: number;
  timestamp: string;
  severity: "critical" | "high" | "medium" | "low";
  event_type: string;
  label: string;
  agent: string;
}

export interface Settings {
  theme: string;
  language: string;
  start_with_system: boolean;
  compact_mode: boolean;
}

/**
 * 浏览器降级（非 Tauri）：返回零值而非 mock 假数据。
 * 真实部署只在 Tauri 中跑；浏览器仅用于 UI 开发预览。
 */
const EMPTY_STATUS: StatusInfo = {
  version: "2.0.0-alpha.0",
  protected: true,
  agents: 0,
  mcp_servers: 0,
  today_requests: 0,
  today_blocks: 0,
  today_cost_usd: 0,
  budget_limit_usd: 0,
  proxy_port: 19111,
  ca_trusted: false,
  kill_switch: false,
  uptime_sec: 0,
  bytes_in: 0,
  bytes_out: 0,
  last_sync_unix: 0,
};

export const ipc = {
  async getStatus(): Promise<StatusInfo> {
    if (!inTauri) return EMPTY_STATUS;
    return invoke<StatusInfo>("get_status");
  },
  async getSettings(): Promise<Settings> {
    if (!inTauri) return { theme: "paper", language: "zh", start_with_system: false, compact_mode: false };
    return invoke<Settings>("get_settings");
  },
  async setTheme(theme: string): Promise<void> {
    if (!inTauri) return;
    return invoke("set_theme", { theme });
  },
  async listRecentEvents(): Promise<InterceptEvent[]> {
    if (!inTauri) return [];
    return invoke<InterceptEvent[]>("list_recent_events");
  },
  async triggerKillSwitch(activate: boolean): Promise<boolean> {
    if (!inTauri) return activate;
    return invoke<boolean>("trigger_kill_switch", { activate });
  },
};
