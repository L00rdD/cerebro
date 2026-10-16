// Display helpers. `humanizeDuration` has a Rust twin in crates/cerebro-domain/src/time.rs (exercise 1).

export function humanizeDuration(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  if (s < 60) return `${s}s`;
  if (s < 3600) return `${Math.floor(s / 60)}min`;
  if (s < 86400) return `${Math.floor(s / 3600)}h${String(Math.floor((s % 3600) / 60)).padStart(2, "0")}`;
  return `${Math.floor(s / 86400)}d`;
}

export function shortenHome(path: string): string {
  return path.replace(/^\/(Users|home)\/[^/]+/, "~");
}

export const STATUS_LABELS: Record<string, string> = { busy: "busy", idle: "idle", waiting: "needs you" };
