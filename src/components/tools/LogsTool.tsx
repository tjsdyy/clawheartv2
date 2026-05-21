import { useState } from "react";
import { cn } from "@/lib/utils";
import { useRequestLogs } from "@/hooks/useRequestLogs";

export function LogsTool() {
  const [search, setSearch] = useState("");
  const { data: logs = [], isLoading } = useRequestLogs(50);

  const filtered = logs.filter(
    (r) =>
      !search ||
      r.agent_id?.toLowerCase().includes(search.toLowerCase()) ||
      r.endpoint.toLowerCase().includes(search.toLowerCase()),
  );

  return (
    <div className="flex flex-col h-full">
      <div className="px-6 py-3 border-b border-border-soft flex items-center gap-3 flex-shrink-0">
        <input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="搜索 agent / endpoint / 状态码…"
          className="flex-1 max-w-md bg-bg border border-border-soft rounded-lg px-3 py-1.5 text-[13px] outline-none focus:border-accent transition-colors"
        />
        <span className="text-[11px] font-mono text-text-muted">
          {isLoading ? "加载中…" : `共 ${filtered.length} 条 · 显示最近 50 条`}
        </span>
      </div>

      <div className="flex-1 overflow-auto font-mono text-[11.5px]">
        <table className="w-full">
          <thead className="sticky top-0 bg-bg/75 backdrop-blur-sm z-10">
            <tr className="text-[10px] uppercase tracking-wider text-text-muted font-bold">
              <Th>TIME</Th>
              <Th>AGENT</Th>
              <Th>ENDPOINT</Th>
              <Th right>RES</Th>
              <Th right>↓ IN</Th>
              <Th right>↑ OUT</Th>
              <Th right>LAT (ms)</Th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((r) => (
              <tr
                key={r.id}
                className={cn(
                  "border-b border-border-soft hover:bg-bg-elev cursor-pointer",
                  r.blocked && "bg-critical/5",
                )}
              >
                <Td muted>{r.timestamp}</Td>
                <Td>{r.agent_id ?? "—"}</Td>
                <Td dim>{r.endpoint}</Td>
                <Td right strong>
                  <span
                    className={cn(
                      r.status_code === 200 && "text-accent",
                      r.blocked && "text-critical",
                      r.status_code === 429 && "text-high",
                    )}
                  >
                    {r.blocked ? "BLK" : r.status_code}
                  </span>
                </Td>
                <Td right dim>{formatBytes(r.bytes_in)}</Td>
                <Td right dim>{formatBytes(r.bytes_out)}</Td>
                <Td right dim>{r.latency_ms}</Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function Th({ children, right }: { children: React.ReactNode; right?: boolean }) {
  return <th className={cn("px-4 py-2.5 border-b border-border", right && "text-right")}>{children}</th>;
}

function Td({
  children,
  right,
  dim,
  muted,
  strong,
}: {
  children: React.ReactNode;
  right?: boolean;
  dim?: boolean;
  muted?: boolean;
  strong?: boolean;
}) {
  return (
    <td
      className={cn(
        "px-4 py-2",
        right && "text-right",
        dim && "text-text-dim",
        muted && "text-text-muted",
        strong && "font-bold",
      )}
    >
      {children}
    </td>
  );
}

function formatBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)}M`;
  if (n >= 1024) return `${(n / 1024).toFixed(1)}K`;
  return `${n}B`;
}
