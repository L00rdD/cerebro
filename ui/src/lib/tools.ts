// What the UI says about each AI tool. Everything else stays tool-agnostic.

const REMOTE_HINTS: Record<string, string> = {
  "claude-code": "Remote Control is off: run /remote-control in this session to take over from here.",
};

/** Why a session has no remote link, and how to get one. */
export function remoteHint(tool: string): string {
  return REMOTE_HINTS[tool] ?? "This session can't be taken over from another device.";
}
