import { create } from "zustand";

interface OverlayStore {
  cmdkOpen: boolean;
  trayOpen: boolean;
  themePickerOpen: boolean;
  openCmdk: () => void;
  closeCmdk: () => void;
  toggleCmdk: () => void;
  toggleTray: () => void;
  closeTray: () => void;
  toggleThemePicker: () => void;
  closeAll: () => void;
}

export const useOverlays = create<OverlayStore>((set) => ({
  cmdkOpen: false,
  trayOpen: false,
  themePickerOpen: false,
  openCmdk: () => set({ cmdkOpen: true, trayOpen: false, themePickerOpen: false }),
  closeCmdk: () => set({ cmdkOpen: false }),
  toggleCmdk: () => set((s) => ({ cmdkOpen: !s.cmdkOpen, trayOpen: false, themePickerOpen: false })),
  toggleTray: () => set((s) => ({ trayOpen: !s.trayOpen, cmdkOpen: false, themePickerOpen: false })),
  closeTray: () => set({ trayOpen: false }),
  toggleThemePicker: () => set((s) => ({ themePickerOpen: !s.themePickerOpen, cmdkOpen: false, trayOpen: false })),
  closeAll: () => set({ cmdkOpen: false, trayOpen: false, themePickerOpen: false }),
}));
