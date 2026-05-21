import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import { QK } from "@/lib/queryClient";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

/** 打开 ~/.clawheart-v2/auto-backups/skills/ 目录 */
async function openSkillBackupDir() {
  if (!inTauri) {
    toast.info("浏览器预览：跳过实际打开目录");
    return;
  }
  try {
    const dir = await invoke<string>("get_skill_backup_dir");
    await shellOpen(dir);
  } catch (e) {
    toast.error(`无法打开备份目录：${e}`);
  }
}

export interface SkillItem {
  slug: string;
  name: string;
  description: string | null;
  safety_label: "safe" | "warn" | "disabled" | "unaudited";
  scan_score: number;
  user_enabled: boolean;
  stars: number | null;
}

const MOCK: SkillItem[] = [
  { slug: "@anthropic/web-fetch", name: "@anthropic/web-fetch",
    description: "浏览器抓取与解析；遵循 robots.txt",
    safety_label: "safe", scan_score: 94, user_enabled: true, stars: 12300 },
  { slug: "@open/postgres-mcp", name: "@open/postgres-mcp",
    description: "本机 PostgreSQL 客户端，支持只读模式",
    safety_label: "warn", scan_score: 78, user_enabled: false, stars: 3100 },
  { slug: "@clawheart/safe-runner", name: "@clawheart/safe-runner",
    description: "沙箱内执行命令；strict capabilities",
    safety_label: "safe", scan_score: 96, user_enabled: false, stars: 8700 },
  { slug: "@some/legacy-mcp", name: "@some/legacy-mcp",
    description: "含 eval(input)，已被供应链扫描阻止",
    safety_label: "disabled", scan_score: 18, user_enabled: false, stars: 124 },
  { slug: "@anthropic/github-mcp", name: "@anthropic/github-mcp",
    description: "官方 GitHub MCP；仓库读写、PR 管理",
    safety_label: "safe", scan_score: 98, user_enabled: true, stars: 24600 },
  { slug: "@community/vision-tools", name: "@community/vision-tools",
    description: "本机图像分析、OCR 与裁剪",
    safety_label: "unaudited", scan_score: 64, user_enabled: false, stars: 412 },
];

export function useSkills(tab: string = "all") {
  return useQuery({
    queryKey: QK.skills(tab),
    queryFn: async () => {
      const all = inTauri ? await invoke<SkillItem[]>("list_skills") : MOCK;
      return filterByTab(all, tab);
    },
    staleTime: 60_000,
  });
}

function filterByTab(skills: SkillItem[], tab: string): SkillItem[] {
  switch (tab) {
    case "installed":   return skills.filter((s) => s.user_enabled);
    case "recommended": return skills.filter((s) => s.scan_score >= 85);
    case "safe":        return skills.filter((s) => s.safety_label === "safe");
    case "latest":      return [...skills].reverse();
    default:            return skills;
  }
}

export function useToggleSkill() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ slug, enabled }: { slug: string; enabled: boolean }) =>
      inTauri ? invoke<void>("toggle_skill", { slug, enabled }) : Promise.resolve(),
    onSuccess: (_, { enabled }) => {
      qc.invalidateQueries({ queryKey: ["skills"] });
      toast.success(enabled ? "已启用" : "已禁用");
    },
    onError: (err) => toast.error(`操作失败：${err}`),
  });
}

// ──────────────────────────────────────────────────────────────────
// 本机技能发现 / 鉴定扫描 / 备份打包
// ──────────────────────────────────────────────────────────────────

export type BindingKind =
  | { kind: "none" }
  | { kind: "real"; path: string; modified_unix: number }
  | { kind: "symlink"; path: string; target: string; points_to_ssot: boolean }
  | { kind: "broken"; path: string; target: string };

export interface AgentBinding {
  agent_name: string;
  binding: BindingKind;
}

export interface DiscoveredSkill {
  id: string;
  name: string;
  description: string | null;
  version: string | null;
  /** SSOT 状态 */
  in_ssot: boolean;
  ssot_path: string | null;
  /** 每个发现 Agent 的绑定 */
  bindings: AgentBinding[];
  has_skill_md: boolean;
  file_count: number;
  total_bytes: number;
  content_hash: string | null;
  /** 向后兼容字段（来自 bindings 计算） */
  source_agent: string;
  source_path: string;
}

/** 该 binding 是否处于"启用"状态（symlink 指向 SSOT） */
export function isBindingEnabled(b: AgentBinding): boolean {
  return b.binding.kind === "symlink" && b.binding.points_to_ssot;
}

/** 该 binding 是否需要警告（real / broken / 非 SSOT symlink） */
export function bindingWarning(b: AgentBinding): "unmanaged" | "broken" | "external" | null {
  switch (b.binding.kind) {
    case "real":
      return "unmanaged";
    case "broken":
      return "broken";
    case "symlink":
      return b.binding.points_to_ssot ? null : "external";
    default:
      return null;
  }
}

const MOCK_DISCOVERED: DiscoveredSkill[] = [
  {
    id: "web-fetch", name: "web-fetch", description: "Browser fetch with robots.txt",
    version: "1.2.0", in_ssot: true,
    ssot_path: "~/.agents/skills/web-fetch",
    bindings: [
      { agent_name: "claude", binding: { kind: "symlink", path: "~/.claude/skills/web-fetch", target: "~/.agents/skills/web-fetch", points_to_ssot: true } },
      { agent_name: "openeva", binding: { kind: "symlink", path: "~/.openeva/skills/web-fetch", target: "~/.agents/skills/web-fetch", points_to_ssot: true } },
    ],
    has_skill_md: true, file_count: 12, total_bytes: 24_000, content_hash: "a3f8b2e44c1d",
    source_agent: "claude", source_path: "~/.agents/skills/web-fetch",
  },
  {
    id: "safe-runner", name: "safe-runner", description: "Sandbox-only command runner",
    version: "2.0.0", in_ssot: false, ssot_path: null,
    bindings: [
      { agent_name: "claude", binding: { kind: "real", path: "~/.claude/skills/safe-runner", modified_unix: 1716000000 } },
    ],
    has_skill_md: true, file_count: 6, total_bytes: 12_000, content_hash: "c4d9e1f8",
    source_agent: "claude", source_path: "~/.claude/skills/safe-runner",
  },
];

export function useDiscoveredSkills() {
  return useQuery({
    queryKey: ["discovered_skills"],
    queryFn: async () =>
      inTauri ? invoke<DiscoveredSkill[]>("discover_local_skills") : MOCK_DISCOVERED,
    staleTime: 30_000,
  });
}

export interface SkillRuleHit {
  rule_id: string;
  description: string;
  why: string;
  example: string;
  remediation: string;
  matched_needles: string[];
}

export interface SkillFinding extends SkillRuleHit {
  match_count: number;
  weighted_deduction: number;
}

export interface LocalSkillScanReport {
  id: string;
  name: string;
  score: number;
  blocked: boolean;
  hard_triggers: SkillRuleHit[];
  findings: SkillFinding[];
}

export function useScanLocalSkill() {
  return useMutation({
    mutationFn: async (id: string): Promise<LocalSkillScanReport> => {
      if (!inTauri) {
        return {
          id, name: id.split("::")[1] ?? id,
          score: 92, blocked: false, hard_triggers: [],
          findings: [
            {
              rule_id: "SK-102",
              description: "代码中疑似硬编码密钥/口令",
              why: "凭据明文写在技能源码或配置里，技能被打包 / 同步 / 备份时一并外泄；攻击者拿到包就能用其凭据冒充用户访问后端。",
              example: 'api_key = "sk-proj-abc123..."\nDB_PASSWORD = "hunter2"',
              remediation: "迁到环境变量 / OS Keychain / dotenv；代码里只保留键名引用。",
              matched_needles: ["api_key", "password"],
              match_count: 2,
              weighted_deduction: 13,
            },
          ],
        };
      }
      return invoke<LocalSkillScanReport>("scan_local_skill", { id });
    },
    onError: (err) => toast.error(`扫描失败：${err}`),
  });
}

export interface BackupResult {
  zip_path: string;
  skill_count: number;
  total_bytes: number;
}

// ──────────────────────────────────────────────────────────────────
// 详情抽屉
// ──────────────────────────────────────────────────────────────────

export interface SkillFile {
  path: string;
  size: number;
  is_dir: boolean;
  depth: number;
}

export interface SkillDetail {
  meta: DiscoveredSkill;
  files: SkillFile[];
  skill_md: string | null;
  readme: string | null;
}

export function useSkillDetail(id: string | null) {
  return useQuery({
    queryKey: ["skill_detail", id],
    enabled: !!id,
    queryFn: async () => {
      if (!inTauri) {
        return {
          meta: {
            id: id ?? "", name: id?.split("::").pop() ?? "demo", description: "Mock detail",
            version: "0.1.0", source_agent: "claude",
            source_path: `~/.claude/skills/${id?.split("::").pop()}`,
            file_count: 4, total_bytes: 8200, has_skill_md: true,
          } as DiscoveredSkill,
          files: [
            { path: "SKILL.md", size: 1820, is_dir: false, depth: 0 },
            { path: "README.md", size: 4400, is_dir: false, depth: 0 },
            { path: "examples", size: 0, is_dir: true, depth: 0 },
            { path: "examples/hello.md", size: 320, is_dir: false, depth: 1 },
          ],
          skill_md: "---\nname: demo\ndescription: Mock detail\nversion: 0.1.0\n---\n\n# Demo skill\n\nThis is mock content (browser preview).",
          readme: null,
        } as SkillDetail;
      }
      return invoke<SkillDetail>("get_local_skill_detail", { id });
    },
    staleTime: 60_000,
  });
}

// ──────────────────────────────────────────────────────────────────
// 备份历史
// ──────────────────────────────────────────────────────────────────

export interface SkillBackupItem {
  id: number;
  created_at: string;
  zip_path: string;
  skill_count: number;
  total_bytes: number;
  skill_ids: string[];
  skill_names: string[];
  zip_exists: boolean;
}

const MOCK_BACKUPS: SkillBackupItem[] = [
  { id: 2, created_at: "2026-05-19 18:42", zip_path: "~/Downloads/clawheart-skills-backup-1747657920.zip",
    skill_count: 3, total_bytes: 28_400, skill_ids: ["claude::a","openeva::b","codex::c"],
    skill_names: ["a","b","c"], zip_exists: true },
  { id: 1, created_at: "2026-05-15 11:10", zip_path: "~/Downloads/clawheart-skills-backup-1747304100.zip",
    skill_count: 1, total_bytes: 9_120, skill_ids: ["claude::a"], skill_names: ["a"], zip_exists: false },
];

export function useSkillBackups() {
  return useQuery({
    queryKey: ["skill_backups"],
    queryFn: async () =>
      inTauri ? invoke<SkillBackupItem[]>("list_skill_backups") : MOCK_BACKUPS,
    staleTime: 15_000,
  });
}

// ──────────────────────────────────────────────────────────────────
// SSOT 管理（Phase B）
// ──────────────────────────────────────────────────────────────────

export interface SsotConfig {
  path: string;
  exists: boolean;
  total_skills: number;
  total_bytes: number;
}

export function useSsotConfig() {
  return useQuery({
    queryKey: ["ssot_config"],
    queryFn: async () =>
      inTauri
        ? invoke<SsotConfig>("get_ssot_config")
        : ({ path: "~/.agents/skills", exists: true, total_skills: 0, total_bytes: 0 } as SsotConfig),
    staleTime: 5_000,
  });
}

/**
 * 把后端错误字符串转成用户友好提示。
 * Windows 上 symlink 失败给出开发者模式指引。
 */
function explainSkillError(err: unknown, action: string): { msg: string; description?: string } {
  const raw = typeof err === "string" ? err : err instanceof Error ? err.message : String(err);
  const lower = raw.toLowerCase();
  const isWindows =
    typeof navigator !== "undefined" && /Win/.test(navigator.userAgent ?? "");

  if (
    isWindows &&
    (lower.includes("symlink") || lower.includes("permission") || lower.includes("权限"))
  ) {
    return {
      msg: `${action}失败：Windows 需要启用开发者模式`,
      description:
        "「设置 → 隐私和安全性 → 开发者选项」打开后即可创建 symlink；或以管理员身份运行 ClawHeart。",
    };
  }
  if (lower.includes("already") || lower.includes("已存在")) {
    return {
      msg: `${action}失败：目标位置已存在`,
      description: raw,
    };
  }
  return { msg: `${action}失败：${raw}` };
}

export function useMoveToSsot() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string): Promise<DiscoveredSkill | null> => {
      if (!inTauri) {
        await new Promise((r) => setTimeout(r, 200));
        return null;
      }
      return invoke<DiscoveredSkill>("move_skill_to_ssot", { id });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["discovered_skills"] });
      qc.invalidateQueries({ queryKey: ["ssot_config"] });
      toast.success("已迁入集中库", {
        description: "原内容已 zip 备份到 ~/.clawheart-v2/auto-backups/skills/",
        duration: 6000,
        action: { label: "打开备份目录", onClick: () => openSkillBackupDir() },
      });
    },
    onError: (err) => {
      const e = explainSkillError(err, "迁入");
      toast.error(e.msg, e.description ? { description: e.description, duration: 10000 } : undefined);
    },
  });
}

export function useToggleSkillBinding() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      agent,
      enabled,
    }: {
      id: string;
      agent: string;
      enabled: boolean;
    }): Promise<DiscoveredSkill | null> => {
      if (!inTauri) {
        await new Promise((r) => setTimeout(r, 150));
        return null;
      }
      return invoke<DiscoveredSkill>("toggle_skill_binding", { id, agent, enabled });
    },
    onSuccess: (_, { agent, enabled }) => {
      qc.invalidateQueries({ queryKey: ["discovered_skills"] });
      toast.success(enabled ? `已在 .${agent} 启用` : `已在 .${agent} 移除`);
    },
    onError: (err) => {
      const e = explainSkillError(err, "切换");
      toast.error(e.msg, e.description ? { description: e.description, duration: 10000 } : undefined);
    },
  });
}

export function useUninstallSkill() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      if (!inTauri) {
        await new Promise((r) => setTimeout(r, 200));
        return;
      }
      return invoke<void>("uninstall_skill", { id });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["discovered_skills"] });
      qc.invalidateQueries({ queryKey: ["ssot_config"] });
      toast.success("技能已卸载", {
        description: "原内容已 zip 备份到 ~/.clawheart-v2/auto-backups/skills/",
        duration: 8000,
        action: { label: "打开备份目录", onClick: () => openSkillBackupDir() },
      });
    },
    onError: (err) => {
      const e = explainSkillError(err, "卸载");
      toast.error(e.msg, e.description ? { description: e.description, duration: 10000 } : undefined);
    },
  });
}

export function useRepairBinding() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, agent }: { id: string; agent: string }) =>
      inTauri
        ? invoke<DiscoveredSkill>("repair_skill_binding", { id, agent })
        : Promise.resolve(null),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["discovered_skills"] });
      toast.success("symlink 已修复指向集中库");
    },
    onError: (err) => {
      const e = explainSkillError(err, "修复");
      toast.error(e.msg, e.description ? { description: e.description, duration: 10000 } : undefined);
    },
  });
}

export function useDeleteSkillBackup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: number) =>
      inTauri ? invoke<void>("delete_skill_backup", { id }) : Promise.resolve(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skill_backups"] });
      toast.success("已删除备份记录");
    },
    onError: (err) => toast.error(`删除失败：${err}`),
  });
}

export function useBackupSkills() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      ids,
      outputZipPath,
    }: {
      ids: string[];
      outputZipPath?: string | null;
    }): Promise<BackupResult> => {
      if (!inTauri) {
        await new Promise((r) => setTimeout(r, 400));
        return {
          zip_path: outputZipPath ?? `~/Downloads/clawheart-skills-backup-${Date.now()}.zip`,
          skill_count: ids.length,
          total_bytes: ids.length * 12_000,
        };
      }
      return invoke<BackupResult>("backup_local_skills", {
        ids,
        outputZipPath: outputZipPath ?? null,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["skill_backups"] });
    },
    onError: (err) => toast.error(`备份失败：${err}`),
  });
}
