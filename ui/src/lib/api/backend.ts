// Thin typed wrapper around Tauri commands and events: the only file that talks to Rust.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { FocusDto, HookSetup, SnapshotDto } from "./types";

export const backend = {
  snapshot: () => invoke<SnapshotDto | null>("get_snapshot"),
  focus: (sessionId: string) => invoke<FocusDto>("focus_session", { sessionId }),
  openFolder: (sessionId: string) => invoke<void>("open_folder", { sessionId }),
  hookStatus: () => invoke<HookSetup>("hook_status"),
  installHooks: () => invoke<HookSetup>("install_hooks"),
  autostartEnabled: () => invoke<boolean>("autostart_enabled"),
  setAutostart: (enabled: boolean) => invoke<boolean>("set_autostart", { enabled }),
  resizePanel: (height: number) => invoke<void>("resize_panel", { height }),
  quit: () => invoke<void>("quit"),

  onSnapshot: (handler: (snapshot: SnapshotDto) => void): Promise<UnlistenFn> =>
    listen<SnapshotDto>("snapshot", (event) => handler(event.payload)),
  onPanelVisibility: (handler: (visible: boolean) => void): Promise<UnlistenFn> =>
    listen<boolean>("panel-visibility", (event) => handler(event.payload)),
};
