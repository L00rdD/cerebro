# Architecture

Cerebro follows Clean Architecture (same principles as `flutter_clean_template`, translated to Rust):
the domain is pure and depends on nothing; outer layers depend on it through traits.

```
                ┌──────────────────────────── presentation ─────────────────────────────┐
                │  ui/ (Svelte)                     src-tauri/ (cerebro-app)            │
                │  components, scene renderer  ◄──  commands, events, tray, panel,      │
                │  (display + animation only)  ──►  poller, injector (DI), DTOs         │
                └───────────────────────────────────────┬───────────────────────────────┘
                                                        │ calls use cases
                ┌────────────────────────────── domain ─▼───────────────────────────────┐
                │  crates/cerebro-domain   entities · ports (traits) · strategies ·     │
                │                          use cases · deterministic rng                │
                └───────────────────────────────────────▲───────────────────────────────┘
                                                        │ implements ports
                ┌─────────────────────────────── data ──┴───────────────────────────────┐
                │  crates/cerebro-data     datasources · repositories · services ·      │
                │                          strategies (terminal focus per platform)     │
                └───────────────────────────────────────┬───────────────────────────────┘
                                                        │ shares the file contract
   Claude Code ──hooks──► crates/cerebro-hook ──writes──► ~/.cerebro ◄── crates/cerebro-protocol
```

## Crates

| Crate | Layer | Depends on | Role |
|---|---|---|---|
| `cerebro-protocol` | contract | serde | File formats and paths shared by the hook (writer) and the app (reader). Hook event list. |
| `cerebro-domain` | domain | thiserror | Entities, ports, casting strategy, use cases. No I/O, no serde. |
| `cerebro-data` | data | domain, protocol | Implements every port with real files, processes and AppleScript. |
| `cerebro-hook` | tool | protocol | Tiny binary run by Claude Code on each hook event. Never prints, always exits 0. |
| `cerebro-app` (`src-tauri/`) | presentation | domain, data, Tauri | Wiring, tray, panel window, commands, polling loop. |
| `cerebro-cli` | presentation | domain, data | Exercise playground (see `EXERCISES.md`). |

Template mapping: `repositories/` (abstract) → `ports/`; `usecases/` → `usecases/`;
`dependency_injection/injector.dart` → `src-tauri/src/injector.rs`; BLoC → Svelte state
(`ui/src/lib/state/app.svelte.ts`) fed by backend events.

## Data flow

1. **Claude Code** writes `~/.claude/sessions/<pid>.json` for each interactive session (internal format).
2. **cerebro-hook** (registered in `~/.claude/settings.json`) writes `~/.cerebro/sessions/<id>.json`
   (status, including *waiting for permission*) and `~/.cerebro/agents/<session>/<agent>.json`.
3. **The poller** (`src-tauri/src/poller.rs`) runs `ObserveSessions` every 1 s while the panel
   is open, 2 s otherwise:
   - read the registry, keep sessions whose pid is alive **and** started at the recorded time
     (`ps` in UTC: a recycled pid is rejected);
   - overlay *waiting* from hook records; fall back to hook records only if the registry is
     unreadable, or empty while hooks show recent activity;
   - attach subagents, prune orphans;
   - cast rooms and characters (`strategies/casting.rs`), persisted in `~/.cerebro/assignments.json`.
4. If the snapshot content changed, it is sent to the UI (`snapshot` event) and the tray is updated.
5. **The UI** draws each scene on a canvas from PNG sprite sheets. Wandering is a pure function
   of `(actor id, time)`; one shared 10 fps clock runs only while the panel is visible.

## Assets

Art lives in files, not code:

```
assets/characters/<id>.png     36×24 sheet: 3 frames (stand, step, step) × 2 rows (front, back), 12×12 each
assets/characters/characters.json   id, name, sprite, optional aura color
assets/rooms/<id>.png          96×60 frames side by side (1 frame if static)
assets/rooms/rooms.json        id, name, frames, fps, walkArea
assets/ui/emotes.png           5×5 "z" and "!"
assets/tray/*.png              template icons for the menu bar
```

The backend embeds the manifests (`EmbeddedCatalog`) to cast ids; the UI loads the same
files. The `shipped_manifests_are_valid` test guards them.

## Click to focus

`MacFocuser` walks up the process tree from the Claude pid to the hosting `.app`. If a
`TabSelector` knows that app (iTerm2 today, Terminal.app in exercise 6), it selects the tab
whose tty matches the session; otherwise the app is just activated. Hook-only sessions (no pid)
are not clickable.

## Decisions

- **Tauri** over Electron/Flutter: small footprint for an always-on tray app, Rust backend.
- **Files over sockets** between hook and app: the hook works even when the app is closed,
  and stays a few dozen lines.
- **Registry first, hooks as fallback**: the registry knows sessions opened before Cerebro and
  their names; hooks protect against format changes and add the *waiting* status.
- **Deterministic randomness** everywhere: nothing to store per frame, stable across restarts.
