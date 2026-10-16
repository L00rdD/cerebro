# Cerebro

A menu bar app that shows every running Claude Code session as a room of the X-Mansion,
seen from above. Each session gets a random room and an X-Man; each subagent it spawns adds
another one. They wander around while working and fall asleep when the session is idle.
Click a room to jump to that session's terminal tab.

- 22 rooms (Danger Room, Blackbird Hangar, Cerebro, Beast's Lab…) and 74 characters, stored as PNG + JSON in `assets/`.
- Stable picks: a session keeps its room and cast for its whole life, even across restarts.
- Click to focus: selects the exact iTerm2 tab; activates the hosting app otherwise.
- Shows *needs you* when Claude is waiting for a permission.
- Built with Tauri 2 (Rust + Svelte); macOS today, Windows and Linux on the roadmap.

## Requirements

- Rust (stable) and Node 20+.
- macOS 12+ for now.

## Commands

```sh
npm install
npm run dev            # build the hook sidecar, run the app with hot reload
npm run install:macos  # release build, installed to ~/Applications/Cerebro.app
npm test               # Rust + UI tests
npm run lint:rust      # Clippy (pedantic, warnings are errors)
npm run check          # Svelte / TypeScript checks
```

All Rust commands go through `scripts/with-rust.sh`, which puts a real `cargo` first in `PATH`.

On first launch, click **Install** in the panel banner: it registers `cerebro-hook` in
`~/.claude/settings.json` (a backup is written next to it). Without hooks, sessions still show
up, but subagents and the *waiting* status don't.

## Project layout

```
crates/cerebro-protocol   file contract shared by hook and app
crates/cerebro-domain     entities, ports, use cases (pure Rust)
crates/cerebro-data       adapters: files, processes, AppleScript, settings
crates/cerebro-hook       the Claude Code hook binary
crates/cerebro-cli        command line companion (exercises)
src-tauri/                desktop shell: tray, panel, commands, DI
ui/                       Svelte UI: cards and canvas scenes
assets/                   sprites and manifests
docs/ARCHITECTURE.md      how it fits together
EXERCISES.md              learn Rust by improving Cerebro
```

## Data and privacy

Cerebro only reads local files (`~/.claude/sessions`, `~/.cerebro`) and runs `ps`, `open` and
`osascript`. Nothing leaves the machine.

X-Men characters are Marvel properties; this is a personal fan project, not for distribution.
