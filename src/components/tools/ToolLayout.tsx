import { ReactNode, useState, createContext, useContext } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { cn } from "@/lib/utils";

interface Props {
  title: string;
  tabs?: string[];
  actions?: ReactNode;
  children: ReactNode;
}

interface ToolLayoutCtx {
  activeTab: number;
}
const Ctx = createContext<ToolLayoutCtx>({ activeTab: 0 });

/** Children inside <ToolLayout> can read the active tab index */
export function useToolLayoutTab(): number {
  return useContext(Ctx).activeTab;
}

export function ToolLayout({ title, tabs, actions, children }: Props) {
  const navigate = useNavigate();
  const [active, setActive] = useState(0);

  return (
    <Ctx.Provider value={{ activeTab: active }}>
    <div className="flex flex-col h-full">
      <header
        className="px-4 flex items-center gap-3 bg-transparent flex-shrink-0"
        style={{ height: 44 }}
      >
        <button
          onClick={() => navigate("/")}
          className="flex items-center gap-1.5 px-2 py-1 rounded-md text-text-muted hover:bg-bg-elev/60 hover:text-text text-[12.5px] transition-colors"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
          <span>返回</span>
        </button>
        <span className="opacity-30 text-text-muted">·</span>
        <div className="font-semibold text-[13.5px] tracking-tight">{title}</div>

        {tabs && (
          <div className="flex gap-0.5 ml-3">
            {tabs.map((t, i) => (
              <button
                key={t}
                onClick={() => setActive(i)}
                className={cn(
                  "px-2.5 py-1 rounded-md text-[12px] font-medium transition-colors",
                  active === i
                    ? "bg-bg-elev/70 text-text"
                    : "text-text-muted hover:text-text",
                )}
              >
                {t}
              </button>
            ))}
          </div>
        )}

        <div className="flex-1" />

        {actions}
      </header>

      <div className="flex-1 overflow-auto animate-fadein">
        {children}
      </div>
    </div>
    </Ctx.Provider>
  );
}

export function ToolBarBtn({ icon, title, onClick }: { icon: ReactNode; title: string; onClick?: () => void }) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="w-9 h-9 rounded-lg flex items-center justify-center text-text-dim hover:text-text hover:bg-bg-elev2 transition-colors"
    >
      {icon}
    </button>
  );
}
