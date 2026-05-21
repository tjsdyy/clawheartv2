import { create } from "zustand";
import { persist } from "zustand/middleware";

interface OnboardingStore {
  completed: boolean;
  setCompleted: (v: boolean) => void;
}

export const useOnboarding = create<OnboardingStore>()(
  persist(
    (set) => ({
      completed: false,
      setCompleted: (v) => set({ completed: v }),
    }),
    { name: "clawheart-onboarding" },
  ),
);
