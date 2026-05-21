import { create } from "zustand";
import { persist } from "zustand/middleware";
import { ipc } from "@/lib/ipc";

export type ThemeName = "paper" | "carbon" | "glacier" | "terminal" | "cyber";

export const THEMES: { id: ThemeName; label: string; hint: string; swatch: string }[] = [
  { id: "paper",    label: "Paper · 羊皮纸",    hint: "浅色 / Obsidian Light",     swatch: "linear-gradient(135deg,#F5F2EB 50%,#047857 50%)" },
  { id: "carbon",   label: "Carbon · 深空墨",   hint: "Linear / GitHub Dark",       swatch: "linear-gradient(135deg,#0B0E11 50%,#22C55E 50%)" },
  { id: "glacier",  label: "Glacier · 冰川蓝",  hint: "Tailscale / Big Sur",       swatch: "linear-gradient(135deg,#0F1419 50%,#38BDF8 50%)" },
  { id: "terminal", label: "Terminal · 终端绿", hint: "k9s / btop",                swatch: "linear-gradient(135deg,#000 50%,#00FF88 50%)" },
  { id: "cyber",    label: "Cyber · 赛博紫",    hint: "Vercel / Cyberpunk",        swatch: "linear-gradient(135deg,#0A0612 50%,#C084FC 50%)" },
];

interface ThemeStore {
  theme: ThemeName;
  setTheme: (t: ThemeName) => void;
}

export const useTheme = create<ThemeStore>()(
  persist(
    (set) => ({
      theme: "paper",
      setTheme: (t) => {
        applyTheme(t);
        ipc.setTheme(t).catch(() => {});
        set({ theme: t });
      },
    }),
    {
      name: "clawheart-theme",
      onRehydrateStorage: () => (state) => {
        if (state?.theme) applyTheme(state.theme);
      },
    },
  ),
);

function applyTheme(t: ThemeName) {
  const html = document.documentElement;
  const classes = Array.from(html.classList);
  for (const c of classes) if (c.startsWith("theme-")) html.classList.remove(c);
  html.classList.add(`theme-${t}`);
}
