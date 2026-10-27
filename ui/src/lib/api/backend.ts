// Picks how the UI reaches Cerebro: Tauri inside the desktop panel, HTTP on the phone page.
// Same build for both: the phone is served the very files the panel shows.

import { tauriBackend, type DesktopBackend } from "./desktop";

/** Desktop-only features (focus, hooks, autostart…); `null` on the phone page. */
export const desktop: DesktopBackend | null = "__TAURI_INTERNALS__" in window ? tauriBackend : null;

export { webBackend, Unauthorized } from "./web";
