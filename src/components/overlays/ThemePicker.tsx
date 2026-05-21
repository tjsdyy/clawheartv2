import { useOverlays } from "@/hooks/useOverlays";
import { useTheme, THEMES, type ThemeName } from "@/hooks/useTheme";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

export function ThemePicker() {
  const { themePickerOpen, toggleThemePicker } = useOverlays();
  const { theme, setTheme } = useTheme();

  if (!themePickerOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/40 backdrop-blur-sm z-[500] animate-fadein"
      onClick={toggleThemePicker}
    >
      <div
        className="absolute top-14 right-4 surface w-[280px] overflow-hidden p-2"
        style={{ boxShadow: "0 16px 48px rgb(0 0 0 / 0.4)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="text-[10px] uppercase tracking-wider text-text-muted py-2 px-3 font-bold font-mono">
          外观主题
        </div>
        {THEMES.map((t) => (
          <button
            key={t.id}
            onClick={() => { setTheme(t.id as ThemeName); }}
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors text-left",
              theme === t.id ? "bg-bg-elev2" : "hover:bg-bg-elev2",
            )}
          >
            <span
              className="w-7 h-7 rounded-lg flex-shrink-0 border-2"
              style={{
                background: t.swatch,
                borderColor: theme === t.id ? "rgb(var(--accent))" : "rgb(var(--border))",
              }}
            />
            <div className="flex-1 min-w-0">
              <div className="text-[13px] font-medium truncate">{t.label}</div>
              <div className="text-[11px] text-text-muted truncate">{t.hint}</div>
            </div>
            {theme === t.id && <Check className="w-4 h-4 text-accent flex-shrink-0" />}
          </button>
        ))}
      </div>
    </div>
  );
}
