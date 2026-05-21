import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

export function severityToColor(s: string): string {
  switch (s) {
    case "critical": return "rgb(var(--critical))";
    case "high":     return "rgb(var(--high))";
    case "medium":   return "rgb(var(--medium))";
    case "low":      return "rgb(var(--low))";
    default:         return "rgb(var(--text-muted))";
  }
}
