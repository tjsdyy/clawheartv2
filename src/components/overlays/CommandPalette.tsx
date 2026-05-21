import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, ScanLine, Pause, Activity, Wrench, Users, ClipboardCopy, FileDown, SlidersHorizontal, type LucideIcon } from "lucide-react";
import { useOverlays } from "@/hooks/useOverlays";
import { ipc } from "@/lib/ipc";
import { cn } from "@/lib/utils";

interface Cmd {
  id: string;
  icon: LucideIcon;
  label: string;
  section: string;
  kbd?: string;
  action: () => void;
}

export function CommandPalette() {
  const { cmdkOpen, closeCmdk } = useOverlays();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const cmds: Cmd[] = [
    { id: "scan",     icon: ScanLine, label: "立即扫描本机",          section: "推荐", kbd: "⌘S",     action: () => { navigate("/tools/scan"); closeCmdk(); } },
    { id: "access",   icon: SlidersHorizontal, label: "切换监控模式（基础 / 全量 / 沙箱）", section: "推荐", action: () => { navigate("/tools/access_mode"); closeCmdk(); } },
    { id: "pause",    icon: Pause,    label: "暂停防护 5 分钟",        section: "推荐", kbd: "⌘P",     action: () => closeCmdk() },
    { id: "monitor",  icon: Activity, label: "最近 10 条拦截",         section: "导航", action: () => { navigate("/tools/monitor"); closeCmdk(); } },
    { id: "skills",   icon: Wrench,   label: "打开技能市场",           section: "导航", action: () => { navigate("/tools/skills"); closeCmdk(); } },
    { id: "agents",   icon: Users,    label: "重新发现 Agent",         section: "导航", action: () => closeCmdk() },
    { id: "ca",       icon: ClipboardCopy, label: "复制 CA 证书路径", section: "工具", action: () => { navigator.clipboard.writeText("~/.clawheart/ca/clawheart-ca.pem"); closeCmdk(); } },
    { id: "export",   icon: FileDown, label: "导出今日审计 (JSON)",    section: "工具", action: () => closeCmdk() },
  ];

  const filtered = cmds.filter((c) =>
    c.label.toLowerCase().includes(query.toLowerCase()),
  );

  useEffect(() => {
    if (cmdkOpen) {
      setQuery("");
      setActive(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [cmdkOpen]);

  useEffect(() => {
    if (!cmdkOpen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setActive((a) => Math.min(a + 1, filtered.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setActive((a) => Math.max(a - 1, 0));
      } else if (e.key === "Enter" && filtered[active]) {
        e.preventDefault();
        filtered[active].action();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cmdkOpen, filtered, active]);

  if (!cmdkOpen) return null;

  // Group by section preserving order
  const grouped: Record<string, Cmd[]> = {};
  filtered.forEach((c) => {
    if (!grouped[c.section]) grouped[c.section] = [];
    grouped[c.section].push(c);
  });

  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-md flex items-start justify-center pt-28 z-[500] animate-fadein"
      onClick={closeCmdk}
    >
      <div
        className="w-[560px] surface overflow-hidden shadow-2xl"
        style={{ boxShadow: "0 32px 96px rgb(0 0 0 / 0.4)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="px-5 py-3.5 border-b border-border flex items-center gap-3">
          <Search className="w-[18px] h-[18px] text-text-muted" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => { setQuery(e.target.value); setActive(0); }}
            placeholder="搜索命令、工具或事件…"
            className="flex-1 bg-transparent border-none outline-none text-[15px]"
          />
        </div>

        <div className="p-1.5 max-h-[360px] overflow-auto">
          {Object.entries(grouped).map(([section, items]) => (
            <div key={section}>
              <div className="text-[10px] uppercase tracking-wider text-text-muted py-1.5 px-3 font-bold font-mono">
                {section}
              </div>
              {items.map((c) => {
                const Icon = c.icon;
                const isActive = filtered[active]?.id === c.id;
                return (
                  <button
                    key={c.id}
                    onClick={c.action}
                    onMouseEnter={() => setActive(filtered.findIndex((f) => f.id === c.id))}
                    className={cn(
                      "w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] text-left transition-colors",
                      isActive ? "bg-bg-elev2 text-text" : "text-text-dim hover:bg-bg-elev2 hover:text-text",
                    )}
                    style={isActive ? { boxShadow: "inset 2px 0 0 rgb(var(--accent))" } : {}}
                  >
                    <Icon className="w-[15px] h-[15px]" />
                    <span>{c.label}</span>
                    {c.kbd && <span className="ml-auto kbd">{c.kbd}</span>}
                  </button>
                );
              })}
            </div>
          ))}
          {filtered.length === 0 && (
            <div className="px-3 py-8 text-center text-text-muted text-sm">无匹配命令</div>
          )}
        </div>

        <div className="px-3.5 py-2 border-t border-border flex gap-3.5 text-[10.5px] font-mono text-text-muted">
          <span>↑↓ 选择</span>
          <span>↵ 执行</span>
          <span>esc 关闭</span>
        </div>
      </div>
    </div>
  );
}
