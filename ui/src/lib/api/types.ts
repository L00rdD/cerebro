// Mirrors crates/cerebro-web/src/dto.rs (snapshots) and src-tauri/src/dto.rs (desktop only).
// Keep them in sync.

export type SessionStatus = "busy" | "idle" | "waiting" | (string & {});

export interface ActorDto {
  /** Stable id used to seed the wandering path. */
  id: string;
  character: string;
  /** null for the session lead, the subagent type otherwise. */
  agentKind: string | null;
}

export interface SceneDto {
  id: string;
  /** AI tool the session runs in, e.g. "claude-code". */
  tool: string;
  name: string;
  cwd: string;
  status: SessionStatus;
  statusSince: number;
  startedAt: number;
  focusable: boolean;
  /** Where to continue the session from another device (e.g. Claude Code's Remote Control). */
  remoteUrl: string | null;
  room: string;
  actors: ActorDto[];
}

/** A source that failed or returned degraded data; its message is ready to display. */
export interface WarningDto {
  tool: string;
  message: string;
}

export interface SnapshotDto {
  scenes: SceneDto[];
  warnings: WarningDto[];
  counts: { sessions: number; busy: number; agents: number; characters: number };
  takenAt: number;
}

export type FocusDto = { kind: "tab"; app: string } | { kind: "app"; app: string } | { kind: "notFound" };

export type HookSetup = "installed" | "missing" | "outdated";

/** How a phone reaches Cerebro through the private network (desktop panel only). */
export interface MobileDto {
  /** e.g. "ZeroTier". */
  network: string;
  state: "notInstalled" | "stopped" | "offline" | "online" | "error";
  /** One link per private address, key included. Empty until the server listens. */
  urls: string[];
  message: string | null;
}
