import { ReactNode } from "react";
import { Palette } from "lucide-react";
import { useOverlays } from "@/hooks/useOverlays";
import { useStatus } from "@/hooks/useStatus";

export function AppShell({ children }: { children: ReactNode }) {
  const { toggleTray, toggleThemePicker } = useOverlays();
  const { data: status } = useStatus();

  return (
    <div className="relative flex flex-col h-screen bg-bg overflow-hidden">
      {/* 沉浸式背景层 — radial glow + 网格 */}
      <div className="immersive-bg" aria-hidden />

      {/* 顶部 28px 拖拽区（macOS Overlay 信号灯悬浮其上） */}
      <div
        data-tauri-drag-region
        className="absolute top-0 left-0 right-0 h-7 z-10"
        aria-hidden
      />

      {/* 浮动右上角操作组 — 仅主题切换 */}
      <div className="absolute top-2 right-3 z-30 flex items-center">
        <FloatIcon title="切换主题" onClick={toggleThemePicker}>
          <Palette className="w-4 h-4" />
        </FloatIcon>
      </div>

      {/* Body */}
      <main className="relative flex-1 overflow-auto z-20 pt-7">
        {children}
      </main>

      {/* 极简底栏 */}
      <StatusBar status={status} onTrayClick={toggleTray} />
    </div>
  );
}

function FloatIcon({
  children,
  title,
  onClick,
}: {
  children: ReactNode;
  title: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="w-8 h-8 rounded-lg flex items-center justify-center transition-all text-text-muted hover:text-text hover:bg-bg-elev/60 backdrop-blur-sm"
    >
      {children}
    </button>
  );
}

function StatusBar({
  status,
  onTrayClick,
}: {
  status: ReturnType<typeof useStatus>["data"] | null;
  onTrayClick: () => void;
}) {
  return (
    <footer className="relative z-20 h-6 px-4 flex items-center justify-end gap-2 text-[10.5px] font-mono text-text-muted flex-shrink-0 bg-transparent">
      <button
        onClick={onTrayClick}
        className="flex items-center gap-1.5 hover:text-text-dim transition-colors"
      >
        <span className="live-dot" />
        <span>{status?.protected ? "防护中" : "防护已暂停"}</span>
      </button>
      <Sep />
      <span>:{status?.proxy_port ?? 19111}</span>
      <Sep />
      <span>CA {status?.ca_trusted ? "✓" : "—"}</span>
      <Sep />
      <span className="opacity-70">
        {formatSyncTime(status?.last_sync_unix)} · v{status?.version ?? "2.0.0-alpha.0"}
      </span>
    </footer>
  );
}

function Sep() {
  return <span className="opacity-40">·</span>;
}

function formatSyncTime(unix?: number): string {
  if (!unix || unix === 0) return "单机模式";
  const d = new Date(unix * 1000);
  return `同步 ${d.getHours().toString().padStart(2, "0")}:${d.getMinutes()
    .toString()
    .padStart(2, "0")}`;
}
