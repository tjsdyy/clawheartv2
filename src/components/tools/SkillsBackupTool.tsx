import { useMemo, useState, type ReactNode } from "react";
import {
  Package, Loader2, ShieldCheck, ShieldX, ChevronRight,
  Search, History, Trash2, FolderOpen, Archive, ArrowRight,
  Plus, Save, RefreshCw,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { useToolLayoutTab } from "./ToolLayout";
import { SkillDetailDrawer } from "./SkillDetailDrawer";
import { SkillsFilterSidebar, type FilterState } from "./SkillsFilterSidebar";
import { SkillRow, ScoreBadge } from "./SkillRow";
import { MoveToSsotDialog } from "./MoveToSsotDialog";
import {
  useDiscoveredSkills,
  useScanLocalSkill,
  useBackupSkills,
  useSkillBackups,
  useDeleteSkillBackup,
  useMoveToSsot,
  useSsotConfig,
  isBindingEnabled,
  useAdminSkills,
  useAdminUpsertSkill,
  useAdminDeleteSkill,
  useAdminScanSkill,
  type LocalSkillScanReport,
  type SkillBackupItem,
  type AdminSkill,
  type AdminSkillInput,
} from "@/hooks/useSkills";

/**
 * 技能管理工具
 * Tab 0 = 本机技能（发现 + 选择 + 备份）
 * Tab 1 = 扫描报告（已扫描技能的安全详情）
 * Tab 2 = 备份历史（DB 持久化）
 */
export function SkillsBackupTool() {
  const tab = useToolLayoutTab();
  // 跨 tab 共享：扫描结果 + 当前详情抽屉 id
  const [reports, setReports] = useState<Record<string, LocalSkillScanReport>>({});
  const [detailId, setDetailId] = useState<string | null>(null);
  const drawerScan = useScanLocalSkill();

  async function handleDrawerScan() {
    if (!detailId) return;
    try {
      const r = await drawerScan.mutateAsync(detailId);
      setReports((m) => ({ ...m, [detailId]: r }));
    } catch {
      /* toast in hook */
    }
  }

  let body: React.ReactNode;
  if (tab === 3) body = <AdminSkillsView />;
  else if (tab === 1) body = <ScanReportsView reports={reports} />;
  else if (tab === 2) body = <BackupHistoryView />;
  else body = <DiscoverView reports={reports} setReports={setReports} onShowDetail={setDetailId} />;

  return (
    <>
      {body}
      <SkillDetailDrawer
        id={detailId}
        onClose={() => setDetailId(null)}
        report={detailId ? reports[detailId] : undefined}
        scanning={drawerScan.isPending && drawerScan.variables === detailId}
        onScan={handleDrawerScan}
      />
    </>
  );
}

// ──────────────────────────────────────────────────────────────────
// View 3：后台商店维护
// ──────────────────────────────────────────────────────────────────

const EMPTY_ADMIN_FORM: AdminSkillInput = {
  slug: "",
  name: "",
  description: "",
  version: "0.1.0",
  system_status: "available",
  user_enabled: true,
  safety_label: "unaudited",
  scan_score: 0,
  install_path: "",
  metadata: '{\n  "category": "office",\n  "tags": []\n}',
};

function AdminSkillsView() {
  const { data: skills = [], isLoading, refetch, isFetching } = useAdminSkills();
  const upsert = useAdminUpsertSkill();
  const remove = useAdminDeleteSkill();
  const scan = useAdminScanSkill();
  const [query, setQuery] = useState("");
  const [editingSlug, setEditingSlug] = useState<string | null>(null);
  const [form, setForm] = useState<AdminSkillInput>(EMPTY_ADMIN_FORM);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return skills;
    return skills.filter((s) =>
      [s.slug, s.name, s.description ?? "", s.version ?? "", s.install_path ?? "", s.metadata ?? ""]
        .join("\n")
        .toLowerCase()
        .includes(q),
    );
  }, [skills, query]);

  function edit(skill: AdminSkill) {
    setEditingSlug(skill.slug);
    setForm({
      slug: skill.slug,
      name: skill.name,
      description: skill.description ?? "",
      version: skill.version ?? "",
      system_status: skill.system_status,
      user_enabled: skill.user_enabled,
      safety_label: skill.safety_label,
      scan_score: skill.scan_score,
      install_path: skill.install_path ?? "",
      metadata: skill.metadata ?? "",
    });
  }

  function reset() {
    setEditingSlug(null);
    setForm(EMPTY_ADMIN_FORM);
  }

  async function save() {
    try {
      await upsert.mutateAsync(form);
      setEditingSlug(form.slug);
    } catch {
      /* toast in hook */
    }
  }

  async function deleteCurrent() {
    if (!editingSlug) return;
    if (!window.confirm(`确认删除 ${editingSlug}？这会删除后台商店记录与扫描结果，不会删除本机已安装目录。`)) return;
    try {
      await remove.mutateAsync(editingSlug);
      reset();
    } catch {
      /* toast in hook */
    }
  }

  return (
    <div className="grid h-full overflow-hidden" style={{ gridTemplateColumns: "minmax(360px, 0.95fr) minmax(440px, 1.05fr)" }}>
      <section className="border-r border-border-soft flex flex-col min-w-0">
        <div className="px-5 py-3 border-b border-border-soft flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-muted" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="搜索 slug / 名称 / 描述 / metadata…"
              className="w-full bg-bg-elev/40 border border-border-soft rounded-md pl-7 pr-3 py-1.5 text-[12.5px] outline-none focus:border-accent"
            />
          </div>
          <button
            onClick={() => refetch()}
            className="w-8 h-8 rounded-md border border-border-soft bg-bg-elev/50 grid place-items-center hover:border-text-muted"
            title="刷新"
          >
            <RefreshCw className={cn("w-3.5 h-3.5", isFetching && "animate-spin")} />
          </button>
          <button
            onClick={reset}
            className="h-8 px-2.5 rounded-md border border-border-soft bg-bg-elev/50 flex items-center gap-1.5 text-[12px] hover:border-text-muted"
          >
            <Plus className="w-3.5 h-3.5" />
            新建
          </button>
        </div>

        <div className="flex-1 overflow-auto p-4 space-y-2">
          {isLoading && <div className="text-center py-12 text-text-muted text-[12px]">加载中…</div>}
          {!isLoading && filtered.length === 0 && (
            <div className="text-center py-12 text-text-muted text-[12px]">暂无商店 Skill 记录</div>
          )}
          {filtered.map((skill) => (
            <button
              key={skill.slug}
              onClick={() => edit(skill)}
              className={cn(
                "w-full text-left rounded-xl border p-3 transition-all bg-bg-elev/35 hover:bg-bg-elev/60",
                editingSlug === skill.slug ? "border-accent shadow-sm" : "border-border-soft",
              )}
            >
              <div className="flex items-start gap-3">
                <div className="w-9 h-9 rounded-lg bg-bg-elev border border-border-soft grid place-items-center font-mono text-[11px] text-text-dim">
                  {skill.scan_score}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 min-w-0">
                    <strong className="truncate text-[13px]">{skill.name}</strong>
                    <AdminPill value={skill.safety_label} />
                    <AdminPill value={skill.system_status} muted />
                  </div>
                  <div className="mt-1 font-mono text-[10.5px] text-text-muted truncate">{skill.slug}</div>
                  <p className="mt-1 text-[11.5px] text-text-dim line-clamp-2">{skill.description || "无描述"}</p>
                </div>
              </div>
            </button>
          ))}
        </div>
      </section>

      <section className="min-w-0 overflow-auto p-5">
        <div className="max-w-3xl space-y-4">
          <div>
            <p className="text-[10px] uppercase tracking-[0.18em] text-text-muted font-semibold">Admin Skills</p>
            <h3 className="mt-1 text-[17px] font-semibold">{editingSlug ? "编辑 Skill" : "新建 Skill"}</h3>
            <p className="mt-1 text-[12px] text-text-dim">维护后台商店元数据；保存后会进入 skills 表，可供前台列表、扫描和发布流程使用。</p>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <AdminField label="Slug">
              <input value={form.slug} onChange={(e) => setForm((f) => ({ ...f, slug: e.target.value }))} disabled={!!editingSlug} className="admin-input" />
            </AdminField>
            <AdminField label="名称">
              <input value={form.name} onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))} className="admin-input" />
            </AdminField>
            <AdminField label="版本">
              <input value={form.version ?? ""} onChange={(e) => setForm((f) => ({ ...f, version: e.target.value }))} className="admin-input" />
            </AdminField>
            <AdminField label="评分">
              <input type="number" min={0} max={100} value={form.scan_score ?? 0} onChange={(e) => setForm((f) => ({ ...f, scan_score: Number(e.target.value) }))} className="admin-input" />
            </AdminField>
            <AdminField label="系统状态">
              <select value={form.system_status} onChange={(e) => setForm((f) => ({ ...f, system_status: e.target.value as AdminSkillInput["system_status"] }))} className="admin-input">
                <option value="available">available</option>
                <option value="deprecated">deprecated</option>
                <option value="removed">removed</option>
              </select>
            </AdminField>
            <AdminField label="安全标签">
              <select value={form.safety_label} onChange={(e) => setForm((f) => ({ ...f, safety_label: e.target.value as AdminSkillInput["safety_label"] }))} className="admin-input">
                <option value="unaudited">unaudited</option>
                <option value="safe">safe</option>
                <option value="warn">warn</option>
                <option value="disabled">disabled</option>
              </select>
            </AdminField>
          </div>

          <AdminField label="描述">
            <textarea value={form.description ?? ""} onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))} className="admin-input min-h-[72px] resize-y" />
          </AdminField>

          <AdminField label="安装源 / 仓库 / 包地址">
            <input value={form.install_path ?? ""} onChange={(e) => setForm((f) => ({ ...f, install_path: e.target.value }))} className="admin-input font-mono" />
          </AdminField>

          <AdminField label="Metadata JSON">
            <textarea value={form.metadata ?? ""} onChange={(e) => setForm((f) => ({ ...f, metadata: e.target.value }))} className="admin-input min-h-[150px] resize-y font-mono text-[11px]" spellCheck={false} />
          </AdminField>

          <label className="inline-flex items-center gap-2 text-[12px] text-text-dim">
            <input
              type="checkbox"
              checked={form.user_enabled ?? true}
              onChange={(e) => setForm((f) => ({ ...f, user_enabled: e.target.checked }))}
              className="accent-accent"
            />
            前台可见 / 可安装
          </label>

          <div className="flex items-center gap-2 pt-1">
            <button onClick={save} disabled={upsert.isPending} className="admin-primary">
              {upsert.isPending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Save className="w-3.5 h-3.5" />}
              保存
            </button>
            <button
              onClick={() => editingSlug && scan.mutate(editingSlug)}
              disabled={!editingSlug || scan.isPending}
              className="admin-secondary"
            >
              {scan.isPending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <ShieldCheck className="w-3.5 h-3.5" />}
              扫描元数据
            </button>
            <button onClick={deleteCurrent} disabled={!editingSlug || remove.isPending} className="admin-danger">
              {remove.isPending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Trash2 className="w-3.5 h-3.5" />}
              删除
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

function AdminField({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="block mb-1.5 text-[11px] text-text-muted font-medium">{label}</span>
      {children}
    </label>
  );
}

function AdminPill({ value, muted }: { value: string; muted?: boolean }) {
  const tone =
    value === "safe" || value === "available"
      ? "text-emerald-500 bg-emerald-500/10 border-emerald-500/20"
      : value === "warn" || value === "deprecated"
        ? "text-amber-500 bg-amber-500/10 border-amber-500/20"
        : value === "disabled" || value === "removed"
          ? "text-rose-500 bg-rose-500/10 border-rose-500/20"
          : "text-text-muted bg-bg-elev/70 border-border-soft";
  return (
    <span className={cn("inline-flex h-5 items-center rounded-full border px-2 text-[10px] font-mono", muted ? "opacity-80" : "", tone)}>
      {value}
    </span>
  );
}

// ──────────────────────────────────────────────────────────────────
// View 0：本机技能发现 + 备份
// ──────────────────────────────────────────────────────────────────
function DiscoverView({
  reports,
  setReports,
  onShowDetail,
}: {
  reports: Record<string, LocalSkillScanReport>;
  setReports: React.Dispatch<React.SetStateAction<Record<string, LocalSkillScanReport>>>;
  onShowDetail: (id: string) => void;
}) {
  const { data: skills = [], isLoading, refetch, isFetching } = useDiscoveredSkills();
  const { data: ssotConfig } = useSsotConfig();
  const scan = useScanLocalSkill();
  const backup = useBackupSkills();
  const moveSsot = useMoveToSsot();

  const [filter, setFilter] = useState<FilterState>({
    agent: null,
    status: null,
    scan: null,
  });
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [moveDialogOpen, setMoveDialogOpen] = useState(false);

  const filtered = useMemo(() => {
    let pool = skills;

    if (filter.agent) {
      pool = pool.filter((s) =>
        s.bindings.length > 0
          ? s.bindings.some((b) => b.agent_name === filter.agent)
          : s.source_agent === filter.agent,
      );
    }

    if (filter.status === "managed") {
      pool = pool.filter((s) => s.in_ssot);
    } else if (filter.status === "unmanaged") {
      pool = pool.filter((s) => !s.in_ssot);
    } else if (filter.status === "orphan") {
      pool = pool.filter(
        (s) => s.in_ssot && !s.bindings.some(isBindingEnabled),
      );
    }

    if (filter.scan) {
      pool = pool.filter((s) => {
        const r = reports[s.id];
        if (filter.scan === "unscanned") return !r;
        if (!r) return false;
        if (filter.scan === "critical") return r.blocked || r.score < 50;
        if (filter.scan === "warn") return !r.blocked && r.score >= 50 && r.score < 80;
        if (filter.scan === "safe") return !r.blocked && r.score >= 80;
        return true;
      });
    }

    if (search.trim()) {
      const q = search.toLowerCase();
      pool = pool.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          (s.description ?? "").toLowerCase().includes(q) ||
          s.source_agent.toLowerCase().includes(q) ||
          s.bindings.some((b) => b.agent_name.toLowerCase().includes(q)),
      );
    }
    return pool;
  }, [skills, filter, reports, search]);

  const allSelectedHere = filtered.length > 0 && filtered.every((s) => selected.has(s.id));

  async function handleScanAll() {
    if (filtered.length === 0) return;
    toast.info(`开始扫描 ${filtered.length} 个技能…`);
    for (const s of filtered) {
      try {
        const r = await scan.mutateAsync(s.id);
        setReports((m) => ({ ...m, [s.id]: r }));
      } catch {
        // 单个失败继续
      }
    }
    toast.success(`扫描完成：${filtered.length} 项`);
  }

  async function handleBackup() {
    if (selected.size === 0) {
      toast.error("请先选择要备份的技能");
      return;
    }
    const ids = Array.from(selected);
    try {
      const res = await backup.mutateAsync({ ids });
      toast.success(`备份完成 · ${res.skill_count} 项 · ${formatBytes(res.total_bytes)}`, {
        description: res.zip_path,
        duration: 8000,
      });
    } catch {
      // toast 已在 hook
    }
  }

  const unmanagedSelected = filtered.filter(
    (s) => !s.in_ssot && selected.has(s.id),
  );

  function openBulkMoveDialog() {
    if (unmanagedSelected.length === 0) {
      toast.error("请先选择「散落」的技能（未纳入集中库）");
      return;
    }
    setMoveDialogOpen(true);
  }

  async function confirmBulkMove() {
    let ok = 0;
    let fail = 0;
    for (const s of unmanagedSelected) {
      try {
        await moveSsot.mutateAsync(s.id);
        ok += 1;
      } catch {
        fail += 1;
      }
    }
    setMoveDialogOpen(false);
    if (fail === 0) toast.success(`已迁入 ${ok} 个技能到集中库`);
    else toast.warning(`迁入完成：${ok} 成功 / ${fail} 失败`);
  }

  return (
    <div className="grid h-full" style={{ gridTemplateColumns: "240px 1fr" }}>
      <SkillsFilterSidebar
        skills={skills}
        reports={reports}
        filter={filter}
        onFilterChange={setFilter}
        ssotConfig={ssotConfig}
        onRefetch={() => refetch()}
        fetching={isFetching}
      />

      {/* 右侧：技能列表 + 工具条 */}
      <section className="flex flex-col overflow-hidden">
        {/* 工具条 */}
        <div className="px-5 py-3 flex items-center gap-2 border-b border-border-soft flex-shrink-0">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-muted" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索技能名 / 描述 / Agent…"
              className="w-full bg-bg-elev/40 border border-border-soft rounded-md pl-7 pr-3 py-1.5 text-[12.5px] outline-none focus:border-accent transition-colors"
            />
          </div>

          <span className="text-[11px] font-mono text-text-muted">
            {filtered.length} / {skills.length}
          </span>

          <div className="flex-1" />

          <label className="flex items-center gap-1.5 text-[11.5px] text-text-dim cursor-pointer px-2">
            <input
              type="checkbox"
              checked={allSelectedHere}
              onChange={(e) => {
                setSelected((prev) => {
                  const next = new Set(prev);
                  if (e.target.checked) filtered.forEach((s) => next.add(s.id));
                  else filtered.forEach((s) => next.delete(s.id));
                  return next;
                });
              }}
              className="accent-accent w-3 h-3"
            />
            全选当前视图
          </label>

          <button
            onClick={openBulkMoveDialog}
            disabled={moveSsot.isPending || unmanagedSelected.length === 0}
            title={
              unmanagedSelected.length === 0
                ? "勾选「散落」状态的技能后启用"
                : `将 ${unmanagedSelected.length} 个散落技能迁入集中库`
            }
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-[12px] font-medium bg-amber-500/10 border border-amber-500/30 text-amber-600 hover:bg-amber-500/15 disabled:opacity-40"
          >
            {moveSsot.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <ArrowRight className="w-3 h-3" />}
            迁入集中库
            {unmanagedSelected.length > 0 && (
              <span className="font-mono">({unmanagedSelected.length})</span>
            )}
          </button>

          <button
            onClick={handleScanAll}
            disabled={scan.isPending || filtered.length === 0}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-[12px] font-medium bg-bg-elev/60 border border-border-soft hover:border-text-muted disabled:opacity-50"
          >
            {scan.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <ShieldCheck className="w-3 h-3" />}
            扫描全部
          </button>

          <button
            onClick={handleBackup}
            disabled={backup.isPending || selected.size === 0}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[12px] font-semibold text-white hover:-translate-y-px transition-all disabled:opacity-50 disabled:translate-y-0"
            style={{
              background: "rgb(var(--accent))",
              boxShadow: "0 2px 8px rgb(var(--accent) / 0.2)",
            }}
          >
            {backup.isPending ? <Loader2 className="w-3 h-3 animate-spin" /> : <Package className="w-3 h-3" />}
            备份选中 ({selected.size})
          </button>
        </div>

        {/* 列表 */}
        <div className="flex-1 overflow-auto px-5 py-4">
          {isLoading && (
            <div className="text-center py-12 text-text-muted text-[12px]">扫描中…</div>
          )}
          {!isLoading && filtered.length === 0 && (
            <EmptyDiscover empty={skills.length === 0} />
          )}
          <div className="space-y-2">
            {filtered.map((s) => (
              <SkillRow
                key={s.id}
                skill={s}
                checked={selected.has(s.id)}
                onToggle={(v) =>
                  setSelected((prev) => {
                    const next = new Set(prev);
                    if (v) next.add(s.id);
                    else next.delete(s.id);
                    return next;
                  })
                }
                report={reports[s.id]}
                scanning={scan.isPending && scan.variables === s.id}
                onScan={async () => {
                  try {
                    const r = await scan.mutateAsync(s.id);
                    setReports((m) => ({ ...m, [s.id]: r }));
                  } catch {
                    /* toast in hook */
                  }
                }}
                onShowDetail={() => onShowDetail(s.id)}
              />
            ))}
          </div>
        </div>
      </section>

      {moveDialogOpen && (
        <MoveToSsotDialog
          skills={unmanagedSelected}
          ssotPath={ssotConfig?.path ?? "~/.agents/skills"}
          pending={moveSsot.isPending}
          onConfirm={confirmBulkMove}
          onCancel={() => setMoveDialogOpen(false)}
        />
      )}
    </div>
  );
}

// View 1：扫描报告
// ──────────────────────────────────────────────────────────────────
function ScanReportsView({ reports }: { reports: Record<string, LocalSkillScanReport> }) {
  const entries = Object.values(reports).sort((a, b) => a.score - b.score);

  if (entries.length === 0) {
    return (
      <div className="mx-auto py-12 px-8 text-center" style={{ maxWidth: 720 }}>
        <div
          className="inline-flex w-12 h-12 rounded-xl items-center justify-center mb-3"
          style={{
            background: "color-mix(in srgb, rgb(var(--accent)) 12%, transparent)",
            color: "rgb(var(--accent))",
          }}
        >
          <ShieldCheck className="w-6 h-6" />
        </div>
        <h3 className="text-[14px] font-semibold mb-1.5">尚未扫描任何技能</h3>
        <p className="text-[12px] text-text-dim leading-relaxed max-w-md mx-auto">
          在「本机技能」Tab 中点单个技能的「扫描」按钮，或顶部「扫描全部」批量扫描。
          扫描使用 SkillGuard 规则集（HardTrigger + 加权扣分）。
        </p>
      </div>
    );
  }

  return (
    <div className="mx-auto py-6 px-8 space-y-3" style={{ maxWidth: 980 }}>
      <h2 className="text-[16px] font-semibold tracking-tight mb-1">扫描报告</h2>
      <p className="text-[12px] text-text-dim mb-4">
        共 {entries.length} 项 · 已按得分升序排列（低分优先暴露）
      </p>
      {entries.map((r) => (
        <ReportCard key={r.id} report={r} />
      ))}
    </div>
  );
}

function ReportCard({ report }: { report: LocalSkillScanReport }) {
  const blocked = report.blocked;
  return (
    <div
      className={cn(
        "rounded-xl border p-4",
        blocked
          ? "border-critical/40 bg-critical/[0.05]"
          : report.score >= 80
            ? "border-border-soft"
            : "border-amber-500/30 bg-amber-500/[0.04]",
      )}
    >
      <div className="flex items-center gap-3 mb-2">
        <ScoreBadge report={report} />
        <div className="flex-1">
          <div className="font-mono font-semibold text-[13px]">{report.name}</div>
          <div className="text-[10.5px] text-text-muted font-mono">{report.id}</div>
        </div>
        <ChevronRight className="w-4 h-4 text-text-muted" />
      </div>

      {report.hard_triggers.length > 0 && (
        <div className="mb-2 text-[11.5px]">
          <span className="font-semibold text-critical">硬触发：</span>
          <span className="font-mono text-text-dim">{report.hard_triggers.join(", ")}</span>
        </div>
      )}

      {report.findings.length > 0 ? (
        <div className="space-y-1 text-[11px] font-mono">
          {report.findings.map((f, i) => (
            <div key={i} className="flex items-start gap-2">
              <span className="text-text-muted">{f.rule_id}</span>
              <span className="flex-1 text-text-dim">{f.description}</span>
              <span className="text-amber-500">-{f.weighted_deduction}</span>
              <span className="text-text-muted">×{f.match_count}</span>
            </div>
          ))}
        </div>
      ) : !blocked && report.hard_triggers.length === 0 ? (
        <div className="text-[11.5px] text-emerald-500">未触发任何规则</div>
      ) : null}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// View 2：备份历史
// ──────────────────────────────────────────────────────────────────
function BackupHistoryView() {
  const { data: backups = [], isLoading } = useSkillBackups();
  const del = useDeleteSkillBackup();

  if (isLoading) {
    return <div className="text-center py-12 text-text-muted text-[12px]">加载中…</div>;
  }

  if (backups.length === 0) {
    return (
      <div className="mx-auto py-12 px-8 text-center" style={{ maxWidth: 720 }}>
        <div
          className="inline-flex w-12 h-12 rounded-xl items-center justify-center mb-3"
          style={{
            background: "color-mix(in srgb, rgb(var(--accent)) 12%, transparent)",
            color: "rgb(var(--accent))",
          }}
        >
          <History className="w-6 h-6" />
        </div>
        <h3 className="text-[14px] font-semibold mb-1.5">尚无备份历史</h3>
        <p className="text-[12px] text-text-dim leading-relaxed max-w-md mx-auto">
          在「本机技能」Tab 中选中技能并点击「备份选中」后，
          记录会出现在这里 · 保留 zip 路径、文件清单、大小、时间戳。
        </p>
      </div>
    );
  }

  return (
    <div className="mx-auto py-6 px-8 space-y-3" style={{ maxWidth: 980 }}>
      <div className="flex items-end justify-between mb-2">
        <div>
          <h2 className="text-[16px] font-semibold tracking-tight mb-0.5">备份历史</h2>
          <p className="text-[11.5px] text-text-dim">
            共 {backups.length} 条 · 按时间倒序
          </p>
        </div>
      </div>

      {backups.map((b) => (
        <BackupRow key={b.id} backup={b} onDelete={() => del.mutate(b.id)} />
      ))}
    </div>
  );
}

function BackupRow({
  backup,
  onDelete,
}: {
  backup: SkillBackupItem;
  onDelete: () => void;
}) {
  const missing = !backup.zip_exists;
  return (
    <div
      className={cn(
        "rounded-lg border px-4 py-3",
        missing
          ? "border-border-soft bg-bg-elev/10 opacity-70"
          : "border-border-soft bg-bg-elev/20",
      )}
    >
      <div className="flex items-center gap-3 mb-2">
        <div
          className="w-9 h-9 rounded-lg flex items-center justify-center flex-shrink-0"
          style={{
            background: "color-mix(in srgb, rgb(var(--accent)) 12%, transparent)",
            color: "rgb(var(--accent))",
          }}
        >
          <Archive className="w-4 h-4" />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-mono text-[12.5px] font-semibold">
              {backup.skill_count} 项
            </span>
            <span className="opacity-40">·</span>
            <span className="font-mono text-[12px] text-text-dim">
              {formatBytes(backup.total_bytes)}
            </span>
            <span className="opacity-40">·</span>
            <span className="font-mono text-[11.5px] text-text-muted">
              {backup.created_at}
            </span>
            {missing && (
              <span className="text-[10px] font-mono text-amber-500 px-1.5 py-0.5 rounded bg-amber-500/10">
                zip 已删除
              </span>
            )}
          </div>
          <div className="font-mono text-[10.5px] text-text-muted truncate mt-0.5" title={backup.zip_path}>
            {backup.zip_path}
          </div>
        </div>

        <button
          onClick={() => {
            navigator.clipboard.writeText(backup.zip_path);
            toast.success("已复制 zip 路径");
          }}
          className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-border-soft hover:border-text-muted text-text-dim"
          disabled={missing}
        >
          <FolderOpen className="w-3 h-3" />
          复制路径
        </button>
        <button
          onClick={onDelete}
          className="flex items-center gap-1 text-[11px] px-2 py-1 rounded border border-critical/30 text-critical hover:bg-critical/5"
          title="删除该备份记录（不删除 zip）"
        >
          <Trash2 className="w-3 h-3" />
          删记录
        </button>
      </div>

      {backup.skill_names.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-1">
          {backup.skill_names.slice(0, 8).map((n, i) => (
            <span
              key={i}
              className="text-[10.5px] font-mono text-text-dim px-1.5 py-0.5 rounded bg-bg-elev/40"
            >
              {n}
            </span>
          ))}
          {backup.skill_names.length > 8 && (
            <span className="text-[10.5px] font-mono text-text-muted">
              +{backup.skill_names.length - 8}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

function EmptyDiscover({ empty }: { empty: boolean }) {
  return (
    <div className="text-center py-16">
      <div
        className="inline-flex w-12 h-12 rounded-xl items-center justify-center mb-3"
        style={{
          background: "color-mix(in srgb, rgb(var(--tool-skills)) 12%, transparent)",
          color: "rgb(var(--tool-skills))",
        }}
      >
        <Package className="w-6 h-6" />
      </div>
      <h3 className="text-[14px] font-semibold mb-1.5">
        {empty ? "未发现本机技能" : "当前过滤无匹配"}
      </h3>
      <p className="text-[12px] text-text-dim leading-relaxed max-w-sm mx-auto">
        {empty
          ? "ClawHeart 会扫描 ~/.<agent>/skills/ 目录；当前未发现任何技能子目录。安装 Claude Code / OpenEva / Codex 等并放入技能后再「重新扫描」。"
          : "调整搜索词或切换 Agent 过滤"}
      </p>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────
// utils
// ──────────────────────────────────────────────────────────────────
function formatBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(2)} MB`;
  if (n >= 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${n} B`;
}
