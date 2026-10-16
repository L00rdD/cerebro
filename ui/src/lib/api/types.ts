// Mirrors src-tauri/src/dto.rs. Keep both files in sync.

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
  name: string;
  cwd: string;
  status: SessionStatus;
  statusSince: number;
  startedAt: number;
  focusable: boolean;
  room: string;
  actors: ActorDto[];
}

export type ModeDto = { kind: "registry" } | { kind: "hooksFallback"; reason: string };

export interface SnapshotDto {
  scenes: SceneDto[];
  mode: ModeDto;
  counts: { sessions: number; busy: number; agents: number; characters: number };
  takenAt: number;
}

export type FocusDto = { kind: "tab"; app: string } | { kind: "app"; app: string } | { kind: "notFound" };

export type HookSetup = "installed" | "missing" | "outdated";
