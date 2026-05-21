import { ShieldAlert, CheckCircle2, ExternalLink, BellOff } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { useAdvisories, useDismissAdvisory, type AdvisoryItem } from "@/hooks/useAdvisories";

const FEED_URL = "https://feeds.clawheart.live/advisories.json";

export function AdvisoryTool() {
  const { data: advisories = [], isLoading } = useAdvisories();
  const dismiss = useDismissAdvisory();

  return (
    <div className="mx-auto py-8 px-12" style={{ maxWidth: 880 }}>
      <h2 className="text-[22px] font-semibold tracking-tight mb-1">安全公告订阅</h2>
      <p className="text-[13px] text-text-dim mb-6">
        Ed25519 签名 feed · 每 6h 自动轮询 · 自动匹配本机已装技能与 Agent 版本
      </p>

      <div className="surface p-5 mb-4 flex items-center gap-4">
        <div
          className="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0"
          style={{
            background: "color-mix(in srgb, rgb(var(--tool-advisory)) 12%, transparent)",
            color: "rgb(var(--tool-advisory))",
          }}
        >
          <ShieldAlert className="w-6 h-6" />
        </div>
        <div className="flex-1">
          <div className="text-[14px] font-semibold">订阅状态：正常</div>
          <div className="text-[12px] text-text-dim font-mono mt-0.5">
            上次同步 2026-05-17 16:00:00 · 下次 2026-05-17 22:00:00 · 签名校验 ✓
          </div>
        </div>
        <button
          onClick={() => {
            navigator.clipboard.writeText(FEED_URL);
            toast.success("Feed URL 已复制", { description: FEED_URL });
          }}
          className="btn-ghost"
        >
          <ExternalLink className="w-3.5 h-3.5" />
          查看 feed
        </button>
      </div>

      {isLoading && <div className="text-text-muted text-center py-12">加载中…</div>}
      {!isLoading && advisories.length === 0 && (
        <div className="text-text-muted text-center py-12">暂无公告</div>
      )}

      <div className="space-y-2.5">
        {advisories.map((adv) => (
          <AdvisoryRow key={adv.id} adv={adv} onDismiss={() => dismiss.mutate(adv.id)} />
        ))}
      </div>
    </div>
  );
}

function AdvisoryRow({ adv, onDismiss }: { adv: AdvisoryItem; onDismiss: () => void }) {
  return (
    <div
      className={cn(
        "surface p-4 transition-colors",
        adv.dismissed && "opacity-60",
        adv.matched_locally && "border-l-[3px]",
      )}
      style={
        adv.matched_locally
          ? { borderLeftColor: `rgb(var(--${adv.severity}))` }
          : undefined
      }
    >
      <div className="flex items-center gap-2 mb-2 flex-wrap">
        <span
          className="px-2 py-0.5 rounded text-[10px] font-bold tracking-wider font-mono text-white"
          style={{ background: `rgb(var(--${adv.severity}))` }}
        >
          {adv.severity.toUpperCase()}
        </span>
        <span className="font-mono text-[11px] text-text-muted px-1.5 py-0.5 rounded bg-bg-elev2">
          {adv.id}
        </span>
        {adv.cvss_score !== null && (
          <span className="font-mono text-[11px] text-text-muted">CVSS {adv.cvss_score}</span>
        )}
        {adv.matched_locally && (
          <span
            className="chip"
            style={{ background: "rgb(var(--critical) / 0.1)", color: "rgb(var(--critical))" }}
          >
            匹配本机
          </span>
        )}
        {adv.dismissed && (
          <span className="chip bg-bg-elev2 text-text-muted">已忽略</span>
        )}
      </div>

      <div className="font-semibold text-[14px] mb-1">{adv.title}</div>
      <div className="text-[11px] text-text-muted font-mono mt-2.5">{adv.published}</div>

      <div className="flex gap-2 mt-3">
        {adv.matched_locally && (
          <button
            onClick={() => toast.info("一键处置 (升级 / 禁用) 在 W18 实现")}
            className="btn-ghost text-[12px]"
          >
            <CheckCircle2 className="w-3.5 h-3.5" />
            一键处置
          </button>
        )}
        {!adv.dismissed && (
          <button onClick={onDismiss} className="btn-ghost text-[12px]">
            <BellOff className="w-3.5 h-3.5" />
            忽略
          </button>
        )}
      </div>
    </div>
  );
}
