import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export type ProtocolKind =
  | "openai"
  | "anthropic"
  | "gemini"
  | "ollama"
  | "openai_responses";

export type ProviderKind =
  | "openrouter"
  | "azure"
  | "deepbricks"
  | "newapi"
  | "openai"
  | "anthropic"
  | "litellm"
  | "custom";

export interface ProviderProfile {
  id: string;
  name: string;
  provider_kind: ProviderKind;
  protocol: ProtocolKind;
  base_url: string;
  default_model: string | null;
  headers: Record<string, string> | null;
  virtual_key: string;
  is_default: boolean;
  enabled: boolean;
  credential_set: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateProfileInput {
  name: string;
  provider_kind: ProviderKind;
  protocol: ProtocolKind;
  base_url: string;
  default_model?: string | null;
  headers?: Record<string, string> | null;
  api_key?: string | null;
  is_default?: boolean;
}

export interface UpdateProfilePatch {
  name: string;
  provider_kind: ProviderKind;
  protocol: ProtocolKind;
  base_url: string;
  default_model?: string | null;
  headers?: Record<string, string> | null;
  enabled: boolean;
}

export interface ConnTestResult {
  ok: boolean;
  latency_ms: number | null;
  status_code: number | null;
  message: string;
}

const QK_PROFILES = ["provider_profiles"] as const;

export function useProviderProfiles() {
  return useQuery({
    queryKey: QK_PROFILES,
    queryFn: async (): Promise<ProviderProfile[]> => {
      if (!inTauri) return [];
      return invoke<ProviderProfile[]>("list_provider_profiles");
    },
    staleTime: 30 * 1000,
  });
}

export function useCreateProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateProfileInput): Promise<ProviderProfile> => {
      if (!inTauri) {
        return makeFakeProfile(input);
      }
      return invoke<ProviderProfile>("create_provider_profile", { input });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_PROFILES });
    },
    onError: (err) => {
      toast.error(`创建失败：${err}`);
    },
  });
}

export function useUpdateProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      patch,
    }: {
      id: string;
      patch: UpdateProfilePatch;
    }): Promise<ProviderProfile> => {
      if (!inTauri) return makeFakeProfile({ ...patch });
      return invoke<ProviderProfile>("update_provider_profile", { id, patch });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_PROFILES });
    },
    onError: (err) => {
      toast.error(`保存失败：${err}`);
    },
  });
}

export function useDeleteProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string): Promise<boolean> => {
      if (!inTauri) return true;
      return invoke<boolean>("delete_provider_profile", { id });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_PROFILES });
    },
    onError: (err) => {
      toast.error(`删除失败：${err}`);
    },
  });
}

export function useSetDefaultProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string): Promise<ProviderProfile> => {
      if (!inTauri) return makeFakeProfile({ name: "fallback" });
      return invoke<ProviderProfile>("set_default_provider_profile", { id });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_PROFILES });
    },
  });
}

export function useSetProviderCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      profileId,
      apiKey,
    }: {
      profileId: string;
      apiKey: string;
    }): Promise<boolean> => {
      if (!inTauri) return true;
      return invoke<boolean>("set_provider_credential", {
        profileId,
        apiKey,
      });
    },
    onSuccess: (_ok, variables) => {
      qc.setQueryData<ProviderProfile[]>(QK_PROFILES, (profiles) =>
        profiles?.map((profile) =>
          profile.id === variables.profileId
            ? { ...profile, credential_set: true }
            : profile,
        ),
      );
      qc.invalidateQueries({ queryKey: QK_PROFILES });
    },
    onError: (err) => {
      toast.error(`凭据保存失败：${err}`);
    },
  });
}

export function useClearProviderCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (profileId: string): Promise<boolean> => {
      if (!inTauri) return true;
      return invoke<boolean>("clear_provider_credential", { profileId });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK_PROFILES });
    },
  });
}

// ──────────────────────────────────────────────────────────────────
// Import from discovered Agents (W6.5)
// ──────────────────────────────────────────────────────────────────

export interface ImportCandidate {
  candidate_id: string;
  suggested_name: string;
  inferred_kind: ProviderKind;
  inferred_protocol: ProtocolKind;
  base_url: string;
  api_key_masked: string;
  source_agents: string[];
  source_labels: string[];
  conflicts_with_existing_profile: boolean;
  existing_profile_name: string | null;
}

export interface BulkImportResult {
  created: ProviderProfile[];
  skipped: { candidate_id: string; reason: string }[];
}

const QK_IMPORT_CANDIDATES = ["import_candidates"] as const;

export function useImportCandidates(enabled: boolean = false) {
  return useQuery({
    queryKey: QK_IMPORT_CANDIDATES,
    queryFn: async (): Promise<ImportCandidate[]> => {
      if (!inTauri) {
        return [
          {
            candidate_id: "imp_demo1",
            suggested_name: "OpenRouter (Cursor)",
            inferred_kind: "openrouter",
            inferred_protocol: "openai",
            base_url: "https://openrouter.ai/api/v1",
            api_key_masked: "sk-or-***xyzd",
            source_agents: ["cursor/Cursor"],
            source_labels: ["Cursor · settings.json[cursor.openaiBaseUrl]"],
            conflicts_with_existing_profile: false,
            existing_profile_name: null,
          },
          {
            candidate_id: "imp_demo2",
            suggested_name: "Anthropic (Claude Code)",
            inferred_kind: "anthropic",
            inferred_protocol: "anthropic",
            base_url: "https://api.anthropic.com",
            api_key_masked: "sk-ant-***wq2v",
            source_agents: ["claude/Claude Code"],
            source_labels: ["Claude Code · settings.json[env.ANTHROPIC_BASE_URL]"],
            conflicts_with_existing_profile: false,
            existing_profile_name: null,
          },
        ];
      }
      return invoke<ImportCandidate[]>("scan_import_candidates");
    },
    enabled,
    staleTime: 0,
  });
}

// 检测到但未能自动导入的 Agent（OAuth / env-var / no-probe / already-proxied）
export interface UnmanagedAgent {
  agent_id: string;
  agent_name: string;
  agent_platform: string;
  reason: "oauth" | "env_var" | "no_config" | "no_probe" | "already_proxied";
  reason_label: string;
  hint: string;
}

export function useUnmanagedAgents(enabled: boolean = false) {
  return useQuery({
    queryKey: ["unmanaged_agents"],
    queryFn: async (): Promise<UnmanagedAgent[]> => {
      if (!inTauri) {
        return [
          {
            agent_id: "claude/Claude Code",
            agent_name: "Claude Code",
            agent_platform: "claude",
            reason: "oauth",
            reason_label: "OAuth 订阅登录",
            hint: "手动建 Profile + 用「一键覆盖」写入 settings.json",
          },
          {
            agent_id: "codex/Codex CLI",
            agent_name: "Codex CLI",
            agent_platform: "codex",
            reason: "env_var",
            reason_label: "环境变量",
            hint: "手动建 Profile + 用「一键覆盖」可在 auth.json / config.toml 强制注入",
          },
        ];
      }
      return invoke<UnmanagedAgent[]>("scan_unmanaged_agents");
    },
    enabled,
    staleTime: 0,
  });
}

export function useBulkImportProfiles() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      candidateIds,
      setFirstAsDefault,
    }: {
      candidateIds: string[];
      setFirstAsDefault?: boolean;
    }): Promise<BulkImportResult> => {
      if (!inTauri) {
        return {
          created: candidateIds.map((id) =>
            makeFakeProfile({ name: `Imported ${id}` }),
          ),
          skipped: [],
        };
      }
      return invoke<BulkImportResult>("bulk_import_profiles", {
        input: {
          candidate_ids: candidateIds,
          set_first_as_default: setFirstAsDefault ?? false,
        },
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["provider_profiles"] });
      qc.invalidateQueries({ queryKey: QK_IMPORT_CANDIDATES });
    },
    onError: (err) => {
      toast.error(`导入失败：${err}`);
    },
  });
}

export async function testProviderConnection(
  profileId: string,
): Promise<ConnTestResult> {
  if (!inTauri) {
    return {
      ok: false,
      latency_ms: null,
      status_code: null,
      message: "浏览器预览：无法连接上游",
    };
  }
  return invoke<ConnTestResult>("test_provider_connection", { profileId });
}

function makeFakeProfile(input: Partial<CreateProfileInput>): ProviderProfile {
  return {
    id: `fake-${Math.random().toString(36).slice(2, 10)}`,
    name: input.name ?? "Demo Profile",
    provider_kind: (input.provider_kind ?? "custom") as ProviderKind,
    protocol: (input.protocol ?? "openai") as ProtocolKind,
    base_url: input.base_url ?? "https://api.example.com/v1",
    default_model: input.default_model ?? null,
    headers: input.headers ?? null,
    virtual_key: `sk-claw-${Math.random().toString(36).slice(2, 18)}`,
    is_default: input.is_default ?? false,
    enabled: true,
    credential_set: !!input.api_key,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

// ──────────────────────────────────────────────────────────────────
// 预设：常见中转站模板
// ──────────────────────────────────────────────────────────────────

export interface ProviderPreset {
  id: ProviderKind;
  label: string;
  protocol: ProtocolKind;
  base_url: string;
  hint?: string;
}

export const PROVIDER_PRESETS: ProviderPreset[] = [
  {
    id: "openrouter",
    label: "OpenRouter",
    protocol: "openai",
    base_url: "https://openrouter.ai/api/v1",
    hint: "聚合多模型，OpenAI 协议兼容",
  },
  {
    id: "openai",
    label: "OpenAI（官方）",
    protocol: "openai",
    base_url: "https://api.openai.com/v1",
  },
  {
    id: "anthropic",
    label: "Anthropic（官方）",
    protocol: "anthropic",
    base_url: "https://api.anthropic.com",
  },
  {
    id: "azure",
    label: "Azure OpenAI",
    protocol: "openai",
    base_url: "https://YOUR-RESOURCE.openai.azure.com/openai/deployments/YOUR-DEPLOY",
    hint: "替换 RESOURCE 与 DEPLOY 占位",
  },
  {
    id: "deepbricks",
    label: "DeepBricks",
    protocol: "openai",
    base_url: "https://api.deepbricks.ai/v1",
  },
  {
    id: "newapi",
    label: "NewAPI / OneAPI",
    protocol: "openai",
    base_url: "https://your-newapi.example.com/v1",
    hint: "自部署聚合网关",
  },
  {
    id: "litellm",
    label: "自托管 LiteLLM",
    protocol: "openai",
    base_url: "http://localhost:4000",
  },
  {
    id: "custom",
    label: "自定义",
    protocol: "openai",
    base_url: "",
  },
];

export function getPreset(id: ProviderKind): ProviderPreset | undefined {
  return PROVIDER_PRESETS.find((p) => p.id === id);
}
