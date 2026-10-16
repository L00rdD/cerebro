// UI state (Svelte 5 runes). Fed by backend events; components only read it.

import { backend } from "../api/backend";
import type { HookSetup, SnapshotDto } from "../api/types";
import type { Catalog } from "../assets/catalog";
import { loadCatalog } from "../assets/catalog";
import { setAnimating } from "../scene/ticker";

class AppState {
  snapshot = $state<SnapshotDto | null>(null);
  catalog = $state<Catalog | null>(null);
  hooks = $state<HookSetup | null>(null);
  autostart = $state(false);
  /** Wall clock refreshed every 15 s so "busy 9min" labels keep moving (report item m1). */
  now = $state(Date.now());
  toast = $state<string | null>(null);
  error = $state<string | null>(null);

  async start() {
    try {
      this.catalog = await loadCatalog();
    } catch (e) {
      this.error = `Assets failed to load: ${String(e)}`;
    }
    await backend.onSnapshot((s) => (this.snapshot = s));
    await backend.onPanelVisibility((visible) => {
      setAnimating(visible);
      if (visible) void this.refreshSettings();
    });
    this.snapshot = await backend.snapshot();
    await this.refreshSettings();
    setInterval(() => (this.now = Date.now()), 15_000);
  }

  async refreshSettings() {
    this.now = Date.now();
    this.hooks = await backend.hookStatus().catch(() => null);
    this.autostart = await backend.autostartEnabled().catch(() => false);
  }

  async installHooks() {
    try {
      this.hooks = await backend.installHooks();
      this.notify("Hooks installed. New Claude sessions will report their subagents.");
    } catch (e) {
      this.notify(`Hook install failed: ${String(e)}`);
    }
  }

  async setAutostart(enabled: boolean) {
    this.autostart = await backend.setAutostart(enabled).catch(() => !enabled);
  }

  notify(message: string) {
    this.toast = message;
    setTimeout(() => {
      if (this.toast === message) this.toast = null;
    }, 3500);
  }
}

export const app = new AppState();
