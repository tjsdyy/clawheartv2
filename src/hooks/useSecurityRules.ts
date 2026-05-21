import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export type RuleKind = "danger" | "injection" | "credential" | "skill" | "audit";

export type DefaultAction = "hard_block" | "block" | "warn" | "weighted" | "skipped";

export type ActionOverride = "block" | "warn" | "skip" | null;

export interface SecurityRuleRow {
  // descriptor 字段（来自代码常量）
  kind: RuleKind;
  id: string;
  category: string | null;
  description: string;
  default_action: DefaultAction;
  pattern_hint: string | null;
  remediation: string | null;
  // 用户覆盖
  enabled: boolean;
  action_override: ActionOverride;
  hits_7d: number;
}

const MOCK_RULES: SecurityRuleRow[] = [
  { kind: "danger", id: "DG-001", category: null,
    description: "rm -rf /", default_action: "block",
    pattern_hint: "\\brm\\s+-rf\\s+/", remediation: null,
    enabled: true, action_override: null, hits_7d: 0 },
  { kind: "danger", id: "DG-003", category: null,
    description: "curl ... | bash", default_action: "block",
    pattern_hint: "\\bcurl\\b[^|]*\\|\\s*(bash|sh|zsh|fish)", remediation: null,
    enabled: true, action_override: null, hits_7d: 12 },
  { kind: "injection", id: "INJ-001", category: "override",
    description: "试图覆盖系统提示", default_action: "block",
    pattern_hint: "ignore previous", remediation: null,
    enabled: true, action_override: null, hits_7d: 3 },
  { kind: "credential", id: "CL-OPENAI", category: "OPENAI_KEY",
    description: "OpenAI API Key 指纹", default_action: "warn",
    pattern_hint: "sk-[A-Za-z0-9]{48,}", remediation: null,
    enabled: true, action_override: null, hits_7d: 0 },
];

export function useSecurityRules() {
  return useQuery({
    queryKey: ["security_rules"],
    queryFn: async () =>
      inTauri ? invoke<SecurityRuleRow[]>("list_security_rules") : MOCK_RULES,
    staleTime: 30_000,
  });
}

export function useToggleRule() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (params: {
      ruleKind: RuleKind;
      ruleId: string;
      enabled: boolean;
    }) => {
      if (!inTauri) return;
      await invoke<void>("toggle_security_rule", {
        ruleKind: params.ruleKind,
        ruleId: params.ruleId,
        enabled: params.enabled,
      });
    },
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["security_rules"] });
      toast.success(vars.enabled ? `已启用 ${vars.ruleId}` : `已禁用 ${vars.ruleId}`);
    },
    onError: (err) => toast.error(`切换失败：${err}`),
  });
}

export function useSetRuleAction() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (params: {
      ruleKind: RuleKind;
      ruleId: string;
      action: "block" | "warn" | "skip" | null;
    }) => {
      if (!inTauri) return;
      await invoke<void>("set_rule_action", {
        ruleKind: params.ruleKind,
        ruleId: params.ruleId,
        action: params.action,
      });
    },
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["security_rules"] });
      toast.success(
        vars.action ? `${vars.ruleId} → ${vars.action.toUpperCase()}` : `${vars.ruleId} 恢复默认动作`,
      );
    },
    onError: (err) => toast.error(`保存失败：${err}`),
  });
}

export function useResetRule() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (params: { ruleKind: RuleKind; ruleId: string }) => {
      if (!inTauri) return;
      await invoke<void>("reset_rule", {
        ruleKind: params.ruleKind,
        ruleId: params.ruleId,
      });
    },
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["security_rules"] });
      toast.success(`已重置 ${vars.ruleId}`);
    },
    onError: (err) => toast.error(`重置失败：${err}`),
  });
}

export function useResetRuleKind() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (ruleKind: RuleKind) => {
      if (!inTauri) return;
      await invoke<void>("reset_rule_kind", { ruleKind });
    },
    onSuccess: (_, ruleKind) => {
      qc.invalidateQueries({ queryKey: ["security_rules"] });
      toast.success(`已重置「${ruleKind}」类全部规则`);
    },
    onError: (err) => toast.error(`重置失败：${err}`),
  });
}

export const KIND_META: Record<RuleKind, { label: string; icon: string }> = {
  danger: { label: "危险指令", icon: "⚠" },
  injection: { label: "提示词注入", icon: "🛡" },
  credential: { label: "凭据指纹", icon: "🔐" },
  skill: { label: "技能供应链", icon: "🧬" },
  audit: { label: "AI 安全审计", icon: "🔍" },
};

export const ACTION_META: Record<DefaultAction, { label: string; color: string }> = {
  hard_block: { label: "HardBlock", color: "rgb(var(--critical))" },
  block:      { label: "Block",     color: "rgb(var(--critical))" },
  warn:       { label: "Warn",      color: "rgb(var(--high))" },
  weighted:   { label: "Weighted",  color: "rgb(var(--accent))" },
  skipped:    { label: "Skipped",   color: "rgb(var(--text-muted))" },
};
