// The phone page's link to Cerebro: the HTTP API the Mac serves on the private network
// (crates/cerebro-web). Read-only: taking over a session goes through its tool's own link.

import type { SnapshotDto } from "./types";

/** The secret carried by the link opened on the phone (`?key=…`). */
const key = new URLSearchParams(location.search).get("key") ?? "";
const RETRY_MS = 3000;

const withKey = (path: string) => `${path}?key=${encodeURIComponent(key)}`;

/** The key is missing, wrong, or was revoked on the Mac. */
export class Unauthorized extends Error {}

export const webBackend = {
  hasKey: key !== "",

  async snapshot(): Promise<SnapshotDto | null> {
    const response = await fetch(withKey("/api/snapshot"), { cache: "no-store" });
    if (response.status === 401) throw new Unauthorized();
    if (response.status === 204) return null;
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return (await response.json()) as SnapshotDto;
  },

  /**
   * Live snapshots over server-sent events. Reconnects by itself after a Wi-Fi switch,
   * a sleeping phone or a restarted Mac. Returns the function that stops following.
   */
  follow(onSnapshot: (snapshot: SnapshotDto) => void, onConnected: (connected: boolean) => void): () => void {
    let source: EventSource | undefined;
    let retry: number | undefined;
    let stopped = false;

    const connect = () => {
      retry = undefined;
      source = new EventSource(withKey("/api/events"));
      source.onopen = () => onConnected(true);
      source.addEventListener("snapshot", (event) => onSnapshot(JSON.parse((event as MessageEvent<string>).data)));
      source.onerror = () => {
        onConnected(false);
        // The browser retries on its own, unless it gave up on this connection for good.
        if (source?.readyState === EventSource.CLOSED && !stopped) retry ??= window.setTimeout(connect, RETRY_MS);
      };
    };
    // iOS freezes pages in the background: reconnect as soon as the page shows again.
    const onVisible = () => {
      if (document.visibilityState !== "visible" || source?.readyState !== EventSource.CLOSED) return;
      window.clearTimeout(retry);
      connect();
    };

    document.addEventListener("visibilitychange", onVisible);
    connect();
    return () => {
      stopped = true;
      source?.close();
      window.clearTimeout(retry);
      document.removeEventListener("visibilitychange", onVisible);
    };
  },
};
