import { useQuery } from "@tanstack/react-query";
import { ipc } from "@/lib/ipc";
import { QK } from "@/lib/queryClient";

export function useStatus() {
  return useQuery({
    queryKey: QK.status,
    queryFn: () => ipc.getStatus(),
    staleTime: 5_000, // status 变化快
    refetchInterval: 10_000,
  });
}

export function useRecentEvents() {
  return useQuery({
    queryKey: QK.events_recent,
    queryFn: () => ipc.listRecentEvents(),
    staleTime: 5_000,
    refetchInterval: 15_000,
  });
}
