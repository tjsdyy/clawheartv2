import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { ipc, type Settings } from "@/lib/ipc";
import { QK } from "@/lib/queryClient";
import { toast } from "sonner";

const inTauri = typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export function useSettings() {
  return useQuery({
    queryKey: QK.settings,
    queryFn: () => ipc.getSettings(),
    staleTime: 5 * 60 * 1000,
  });
}

export function useUpdateSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (settings: Settings) => {
      if (inTauri) {
        await invoke("save_settings", { settings });
      }
      return settings;
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: QK.settings });
    },
    onError: (err) => {
      toast.error("保存失败：" + String(err));
    },
  });
}
