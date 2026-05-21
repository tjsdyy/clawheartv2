import { useState } from "react";
import { Plus, X } from "lucide-react";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { useQueryClient } from "@tanstack/react-query";
import { cn } from "@/lib/utils";
import { useBudgetRules, type BudgetRuleItem } from "@/hooks/useBudget";
import { QK } from "@/lib/queryClient";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

const PROVIDERS = ["global", "anthropic", "openai", "google", "mistral", "groq", "deepseek"];
const PERIODS = ["daily", "monthly"] as const;

export function BudgetTool() {
  const { data: rules = [], isLoading } = useBudgetRules();
  const [adding, setAdding] = useState(false);

  return (
    <div className="mx-auto py-8 px-12" style={{ maxWidth: 820 }}>
      <h2 className="text-[22px] font-semibold tracking-tight mb-1">预算控制</h2>
      <p className="text-[13px] text-text-dim mb-6">
        按 provider × model × period 设置上限 · 80% 告警 · 100% 阻断（失败关闭）
      </p>

      <div className="flex items-center justify-between mb-3">
        <h3 className="text-[13px] font-semibold">规则列表</h3>
        <button
          onClick={() => setAdding((a) => !a)}
          className="btn-ghost text-[12px]"
        >
          {adding ? <X className="w-3.5 h-3.5" /> : <Plus className="w-3.5 h-3.5" />}
          {adding ? "取消" : "新增规则"}
        </button>
      </div>

      {adding && <AddRuleForm onDone={() => setAdding(false)} />}

      {isLoading && <div className="text-text-muted text-center py-12">加载中…</div>}

      {!isLoading && rules.length === 0 && !adding && (
        <div className="surface p-8 text-center text-text-muted text-[13px]">
          暂无预算规则 · 点"新增规则"创建
        </div>
      )}

      <div className="space-y-2">
        {rules.map((r) => (
          <BudgetCard key={r.id} rule={r} />
        ))}
      </div>
    </div>
  );
}

function AddRuleForm({ onDone }: { onDone: () => void }) {
  const [provider, setProvider] = useState("global");
  const [model, setModel] = useState("");
  const [period, setPeriod] = useState<"daily" | "monthly">("daily");
  const [limit, setLimit] = useState(10);
  const [saving, setSaving] = useState(false);
  const qc = useQueryClient();

  async function handleSave() {
    if (limit <= 0) {
      toast.error("上限必须 > 0");
      return;
    }
    setSaving(true);
    try {
      if (inTauri) {
        await invoke("set_budget_rule", {
          rule: {
            provider,
            model: model.trim() ? model.trim() : null,
            period,
            limit_usd: limit,
            enabled: true,
          },
        });
      }
      qc.invalidateQueries({ queryKey: QK.budget });
      toast.success("规则已添加");
      onDone();
    } catch (e) {
      toast.error(`保存失败：${e}`);
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="surface p-5 mb-3 border-l-[3px]" style={{ borderLeftColor: "rgb(var(--accent))" }}>
      <div className="grid grid-cols-2 gap-3 mb-3">
        <Field label="Provider">
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
            className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[13px] outline-none focus:border-accent w-full"
          >
            {PROVIDERS.map((p) => <option key={p} value={p}>{p}</option>)}
          </select>
        </Field>
        <Field label="Model (可选)">
          <input
            value={model}
            onChange={(e) => setModel(e.target.value)}
            placeholder="claude-opus-4-7 / gpt-5.4 ..."
            className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[13px] font-mono outline-none focus:border-accent w-full"
          />
        </Field>
        <Field label="周期">
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value as any)}
            className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[13px] outline-none focus:border-accent w-full"
          >
            {PERIODS.map((p) => <option key={p} value={p}>{p === "daily" ? "每日" : "每月"}</option>)}
          </select>
        </Field>
        <Field label="上限 (USD)">
          <input
            type="number"
            value={limit}
            min={0}
            step={0.5}
            onChange={(e) => setLimit(Number(e.target.value))}
            className="bg-bg-elev2 border border-border rounded-md px-3 py-1.5 text-[13px] font-mono outline-none focus:border-accent w-full"
          />
        </Field>
      </div>
      <div className="flex gap-2 justify-end">
        <button onClick={onDone} className="btn-ghost text-[12px]">取消</button>
        <button
          onClick={handleSave}
          disabled={saving}
          className="btn-primary text-[12px] disabled:opacity-50"
        >
          {saving ? "保存中…" : "保存"}
        </button>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <div className="text-[10px] text-text-muted uppercase tracking-wider font-bold font-mono mb-1">{label}</div>
      {children}
    </label>
  );
}

function BudgetCard({ rule }: { rule: BudgetRuleItem }) {
  const pct = rule.limit_usd > 0 ? (rule.used_usd / rule.limit_usd) * 100 : 0;
  const color =
    pct >= 100 ? "critical" : pct >= 80 ? "high" : pct >= 50 ? "medium" : "low";

  const scope = rule.model
    ? `${rule.provider} · ${rule.model}`
    : rule.provider === "global" ? "全局" : `${rule.provider}`;

  return (
    <div className={cn("surface p-4", !rule.enabled && "opacity-50")}>
      <div className="flex items-center justify-between mb-3">
        <div>
          <div className="text-[14px] font-semibold">{scope}</div>
          <div className="font-mono text-[11px] text-text-muted mt-0.5">
            {rule.period === "daily" ? "每日上限" : "每月上限"}
          </div>
        </div>
        <div className="text-right">
          <div className="font-mono text-[14px] font-semibold">
            ${rule.used_usd.toFixed(2)}{" "}
            <span className="text-text-muted text-[12px]">/ ${rule.limit_usd.toFixed(2)}</span>
          </div>
          <div
            className="font-mono text-[11px] mt-0.5"
            style={{ color: `rgb(var(--${color}))` }}
          >
            {pct.toFixed(1)}%
          </div>
        </div>
      </div>

      <div className="h-1.5 rounded-full bg-bg-elev2 overflow-hidden">
        <div
          className="h-full rounded-full transition-all"
          style={{
            width: `${Math.min(pct, 100)}%`,
            background: `rgb(var(--${color}))`,
          }}
        />
      </div>
    </div>
  );
}
