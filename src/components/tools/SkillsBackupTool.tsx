import { useMemo, useState } from "react";
import {
  Package, Loader2, ShieldCheck, ShieldX, ChevronRight,
  Search, History, Trash2, FolderOpen, Archive, ArrowRight,
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
  type LocalSkillScanReport,
  type SkillBackupItem,
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
  if (tab === 1) body = <ScanReportsView reports={reports} />;
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
