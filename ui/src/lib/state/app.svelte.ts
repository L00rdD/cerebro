// UI state (Svelte 5 runes). Fed by backend events; components only read it.
// The same state drives the desktop panel (Tauri) and the phone page (HTTP).

import { desktop, Unauthorized, webBackend } from "../api/backend";
import type { DesktopBackend } from "../api/desktop";
import type { HookSetup, MobileDto, SnapshotDto } from "../api/types";
import type { Catalog } from "../assets/catalog";
import { loadCatalog } from "../assets/catalog";
import { setAnimating } from "../scene/ticker";

class AppState {
  /** True on the phone page, false in the desktop panel. */
  readonly phone = desktop === null;
  snapshot = $state<SnapshotDto | null>(null);
  catalog = $state<Catalog | null>(null);
  hooks = $state<HookSetup | null>(null);
  autostart = $state(false);
  /** Desktop: how a phone can reach Cerebro. */
  mobile = $state<MobileDto | null>(null);
  /** Phone: whether the live feed from the Mac is up. */
  connected = $state(true);
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
    if (desktop) await this.startDesktop(desktop);
    else await this.startPhone();
    setInterval(() => (this.now = Date.now()), 15_000);
  }

  private async startDesktop(api: DesktopBackend) {
    await api.onSnapshot((s) => (this.snapshot = s));
    await api.onMobile((m) => (this.mobile = m));
    await api.onPanelVisibility((visible) => {
      setAnimating(visible);
      if (visible) void this.refreshSettings();
    });
    this.snapshot = await api.snapshot();
    this.mobile = await api.mobileStatus().catch(() => null);
    await this.refreshSettings();
  }

  private async startPhone() {
    document.documentElement.classList.add("phone");
    const animate = () => {
      setAnimating(document.visibilityState === "visible");
      this.now = Date.now();
    };
    document.addEventListener("visibilitychange", animate);
    animate();

    if (!webBackend.hasKey) {
      this.error = "Open the phone link from Cerebro's panel on your Mac: it carries the access key.";
      return;
    }
    try {
      this.snapshot = await webBackend.snapshot();
    } catch (e) {
      if (e instanceof Unauthorized) {
        this.error = "This link's key is no longer valid. Copy the phone link again from Cerebro's panel.";
        return;
      }
      this.connected = false;
    }
    webBackend.follow(
      (s) => (this.snapshot = s),
      (connected) => (this.connected = connected),
    );
  }

  async refreshSettings() {
    this.now = Date.now();
    this.hooks = (await desktop?.hookStatus().catch(() => null)) ?? null;
    this.autostart = (await desktop?.autostartEnabled().catch(() => false)) ?? false;
  }

  async installHooks() {
    if (!desktop) return;
    try {
      this.hooks = await desktop.installHooks();
      this.notify("Hooks installed. New Claude sessions will report their subagents.");
    } catch (e) {
      this.notify(`Hook install failed: ${String(e)}`);
    }
  }

  async setAutostart(enabled: boolean) {
    if (!desktop) return;
    this.autostart = await desktop.setAutostart(enabled).catch(() => !enabled);
  }

  async startNetwork() {
    if (!desktop) return;
    try {
      await desktop.startNetwork();
    } catch (e) {
      this.notify(`Couldn't start ${this.mobile?.network ?? "the network"}: ${String(e)}`);
    }
  }

  async copyPhoneLink() {
    const url = this.mobile?.urls[0];
    if (!url) return;
    await navigator.clipboard.writeText(url);
    this.notify("Phone link copied. Open it in Safari on your phone, then Share → Add to Home Screen.");
  }

  notify(message: string) {
    this.toast = message;
    setTimeout(() => {
      if (this.toast === message) this.toast = null;
    }, 3500);
  }
}

export const app = new AppState();
