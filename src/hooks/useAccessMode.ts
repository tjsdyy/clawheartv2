import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { QK } from "@/lib/queryClient";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export type AccessTier = "tier1" | "tier2" | "tier3";

export interface AccessModeInfo {
  current_tier: AccessTier;
  reverse_proxy_port: number;
  forward_proxy_port: number;
  ca_installed: boolean;
  ca_path: string;
  system_proxy_active: boolean;
  fetch_url_template: string;
  backend_ready: boolean;
}

export interface CaInstallResult {
  ok: boolean;
  platform: string;
  message: string;
  manual_steps: string[];
  fingerprint?: string | null;
}

export interface CaStatus {
  installed: boolean;
  fingerprint?: string | null;
  expires_at?: string | null;
}

export interface SandboxCommandPreview {
  command: string;
  platform: string;
  feature_available: boolean;
  notes: string[];
}

export interface ProtocolAdapter {
  id: string;
  path: string;
  label: string;
  enabled: boolean;
  default_enabled: boolean;
}

export interface PortUpdateResult {
  ok: boolean;
  tier: AccessTier;
  port: number;
  mode: AccessModeInfo;
}

const FALLBACK_ADAPTERS: ProtocolAdapter[] = [
  { id: "openai_chat", path: "/v1/chat/completions", label: "OpenAI Chat Completions", enabled: true, default_enabled: true },
  { id: "anthropic", path: "/v1/messages", label: "Anthropic Messages", enabled: true, default_enabled: true },
  { id: "openai_responses", path: "/v1/responses", label: "OpenAI Responses (Codex)", enabled: true, default_enabled: true },
  { id: "gemini", path: "/v1beta/models/:generateContent", label: "Google Gemini", enabled: true, default_enabled: true },
  { id: "ollama", path: "/api/chat", label: "Ollama", enabled: true, default_enabled: true },
];

const FALLBACK_ACCESS_MODE: AccessModeInfo = {
  current_tier: "tier1",
  reverse_proxy_port: 19112,
  forward_proxy_port: 19111,
  ca_installed: false,
  ca_path: "~/.clawheart-v2/ca/clawheart-ca.pem",
  system_proxy_active: false,
  fetch_url_template: "http://127.0.0.1:19112/v1",
  backend_ready: false,
};

export function useAccessMode() {
  return useQuery({
    queryKey: QK.access_mode,
    queryFn: async (): Promise<AccessModeInfo> => {
      if (!inTauri) return FALLBACK_ACCESS_MODE;
      return invoke<AccessModeInfo>("get_access_mode");
    },
    staleTime: 30 * 1000,
  });
}

export function useSetAccessMode() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (tier: AccessTier): Promise<AccessModeInfo> => {
      if (!inTauri) return { ...FALLBACK_ACCESS_MODE, current_tier: tier };
      return invoke<AccessModeInfo>("set_access_mode", { tier });
    },
    onSuccess: (data) => {
      qc.setQueryData(QK.access_mode, data);
      qc.invalidateQueries({ queryKey: QK.ca_status });
    },
    onError: (err) => {
      toast.error("切换失败：" + String(err));
    },
  });
}

export function useCaStatus() {
  return useQuery({
    queryKey: QK.ca_status,
    queryFn: async (): Promise<CaStatus> => {
      if (!inTauri) return { installed: false };
      return invoke<CaStatus>("check_ca_status");
    },
    staleTime: 60 * 1000,
  });
}

export function useInstallCa() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (): Promise<CaInstallResult> => {
      if (!inTauri) {
        return {
          ok: false,
          platform: "browser",
          message: "浏览器环境无法安装证书",
          manual_steps: [],
          fingerprint: null,
        };
      }
      return invoke<CaInstallResult>("install_ca");
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK.access_mode });
      qc.invalidateQueries({ queryKey: QK.ca_status });
    },
    onError: (err) => {
      toast.error("CA 安装失败：" + String(err));
    },
  });
}

export function useUninstallCa() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (): Promise<boolean> => {
      if (!inTauri) return false;
      return invoke<boolean>("uninstall_ca");
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK.access_mode });
      qc.invalidateQueries({ queryKey: QK.ca_status });
    },
  });
}

export function useProtocolAdapters() {
  return useQuery({
    queryKey: ["protocol_adapters"] as const,
    queryFn: async (): Promise<ProtocolAdapter[]> => {
      if (!inTauri) return FALLBACK_ADAPTERS;
      return invoke<ProtocolAdapter[]>("list_protocol_adapters");
    },
    staleTime: 60 * 1000,
  });
}

export function useToggleProtocolAdapter() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, enabled }: { id: string; enabled: boolean }): Promise<ProtocolAdapter[]> => {
      if (!inTauri) {
        return FALLBACK_ADAPTERS.map((a) => (a.id === id ? { ...a, enabled } : a));
      }
      return invoke<ProtocolAdapter[]>("toggle_protocol_adapter", { id, enabled });
    },
    onSuccess: (data) => {
      qc.setQueryData(["protocol_adapters"], data);
    },
    onError: (err) => {
      toast.error("切换协议失败：" + String(err));
    },
  });
}

export function useUpdateProxyPort() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      tier,
      port,
    }: {
      tier: AccessTier;
      port: number;
    }): Promise<PortUpdateResult> => {
      if (!inTauri) {
        return {
          ok: true,
          tier,
          port,
          mode: { ...FALLBACK_ACCESS_MODE, [tier === "tier1" ? "reverse_proxy_port" : "forward_proxy_port"]: port } as AccessModeInfo,
        };
      }
      return invoke<PortUpdateResult>("update_proxy_port", { tier, port });
    },
    onSuccess: (result) => {
      qc.setQueryData(QK.access_mode, result.mode);
    },
    onError: (err) => {
      toast.error(`端口校验失败：${err}`);
    },
  });
}

export async function generateSandboxCommand(
  cmd: string,
  args: string[] = [],
): Promise<SandboxCommandPreview> {
  if (!inTauri) {
    return {
      command: `clawheart sandbox -- ${cmd} ${args.join(" ")}`.trim(),
      platform: "browser",
      feature_available: false,
      notes: ["浏览器预览模式 · 仅显示命令字符串"],
    };
  }
  return invoke<SandboxCommandPreview>("generate_sandbox_command", { cmd, args });
}
