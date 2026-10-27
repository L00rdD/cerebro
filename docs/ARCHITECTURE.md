# Architecture

Cerebro follows Clean Architecture (same principles as `flutter_clean_template`, translated to Rust):
the domain is pure and depends on nothing; outer layers depend on it through traits.
It is also agnostic of the AI tool: it sees sessions tagged with a `ToolId`, never where they came from.

```
                ┌──────────────────────────── presentation ─────────────────────────────┐
                │  ui/ (Svelte)                     src-tauri/ (cerebro-app)            │
                │  components, scene renderer  ◄──  commands, events, tray, panel,      │
                │  (display + animation only)  ──►  poller, mobile supervisor, DI       │
                │        ▲ same build                                                   │
                │  phone (Safari) ◄── HTTP + SSE ── crates/cerebro-web (server, DTOs)   │
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
| `cerebro-domain` | domain | thiserror | Entities, ports, casting strategy, use cases. No I/O, no serde, no knowledge of any AI tool. |
| `cerebro-data` | data | domain, protocol | Implements every port with real files, processes and AppleScript. One `SessionSource` per AI tool (`ClaudeCodeSource`). |
| `cerebro-hook` | tool | protocol | Tiny binary run by Claude Code on each hook event. Never prints, always exits 0. |
| `cerebro-app` (`src-tauri/`) | presentation | domain, data, web, Tauri | Wiring, tray, panel window, commands, polling loop, phone server supervisor. |
| `cerebro-cli` | presentation | domain, data | Exercise playground (see `EXERCISES.md`). |
| `cerebro-web` | presentation | domain, axum, tokio | The phone page: serves the UI build, the snapshot API and server-sent events. DTOs shared with the panel. No Tauri, so any host can run it. |

Template mapping: `repositories/` (abstract) → `ports/`; `usecases/` → `usecases/`;
`dependency_injection/injector.dart` → `src-tauri/src/injector.rs`; BLoC → Svelte state
(`ui/src/lib/state/app.svelte.ts`) fed by backend events.

## Data flow

1. **Claude Code** writes `~/.claude/sessions/<pid>.json` for each interactive session (internal format).
2. **cerebro-hook** (registered in `~/.claude/settings.json`) writes `~/.cerebro/sessions/<id>.json`
   (status, including *waiting for permission*) and `~/.cerebro/agents/<session>/<agent>.json`.
3. **The poller** (`src-tauri/src/poller.rs`) runs `ObserveSessions` every 1 s while the panel
   is open, 2 s otherwise. The use case asks every `SessionSource` for its live sessions:
   - **`ClaudeCodeSource`** (`cerebro-data/src/datasources/claude_code.rs`) reads the registry,
     keeps sessions whose pid is alive **and** started at the recorded time (`ps` in UTC: a
     recycled pid is rejected), overlays *waiting* from hook records, falls back to hook records
     only if the registry is unreadable or empty while hooks show recent activity (reported as a
     warning), then attaches subagents and prunes orphans;
   - a failing source becomes a `SourceWarning` and doesn't hide the other sources' sessions;
   - the domain then sorts every session by launch time and casts rooms and characters
     (`strategies/casting.rs`), persisted in `~/.cerebro/assignments.json`.
4. If the snapshot content changed, it is sent to the UI (`snapshot` event) and the tray is updated.
5. **Phones** (`src-tauri/src/mobile.rs`): every 10 s the supervisor asks `ManageNetwork` for the
   private network's state (`ZeroTierNetwork` runs `zerotier-cli -j`). It starts a `WebServer` on
   this machine's addresses on **private** networks, moves it when they change and stops it when
   the network goes away. The poller publishes each new snapshot to the server's `Publisher`;
   every open page gets it as a server-sent event. The API wants the key from the phone link
   (`~/.cerebro/mobile-key`); static files don't.
6. **The UI** draws each scene on a canvas from PNG sprite sheets. Wandering is a pure function
   of `(actor id, time)`: each character has its own way to move (`walk`, `fly`, `teleport`,
   `phase`, `dash`) and a power it uses while pausing between waypoints (`scene/powers.ts`).
   One shared 10 fps clock runs only while the panel (or the phone page) is visible. The same
   build runs in both: `ui/src/lib/api/backend.ts` picks Tauri in the panel, HTTP on the phone,
   where desktop-only features (focus, hooks, autostart) are hidden.

## Assets

Art lives in files, not code:

```
assets/characters/<id>.png     96×72 sheet: 4 frames (stand, step, pass, step) × 3 rows (front, back, side), 24×24 each
assets/characters/characters.json   frame size; id, name, sprite, optional aura color, move, power, powerColor
assets/rooms/<id>.png          192×120 frames side by side (1 frame if static)
assets/rooms/rooms.json        frame size; id, name, frames, fps, walkArea
assets/ui/emotes.png           9×9 "z" and "!"
assets/tray/*.png              template icons for the menu bar
```

The side row faces right; the renderer mirrors it for characters walking left. Scenes are drawn
at 1.5 CSS pixels per art pixel in the panel (3 device pixels on Retina, so the art stays crisp)
and stretched to the screen width on the phone page.

The PNGs are generated by `scripts/art/` (Python + Pillow, deterministic): `characters.py` dresses
a shared body template with each character's look (`looks.py`, features in `features.py`),
`rooms.py` draws every room and writes `rooms.json`, `emotes.py` the emotes. Run them again after
editing a look or a room; hand-drawn PNGs work too, as long as they follow the sizes above.

The backend embeds the manifests (`EmbeddedCatalog`) to cast ids; the UI loads the same
files. The `shipped_manifests_are_valid` test guards them.

## Click to focus

`MacFocuser` walks up the process tree from the Claude pid to the hosting `.app`. If a
`TabSelector` knows that app (iTerm2 today, Terminal.app in exercise 6), it selects the tab
whose tty matches the session; otherwise the app is just activated. Hook-only sessions (no pid)
are not clickable.

## Taking over from the phone

Cerebro reports how to continue a session elsewhere but never drives it: `Session::remote_url` is
filled by the session's source when its tool offers it. For Claude Code it comes from the
registry's `bridgeSessionId`, written while Remote Control is on, and links to
`https://claude.ai/code/session_…`. Another tool fills it its own way, or leaves it `None`.

## Decisions

- **Tauri** over Electron/Flutter: small footprint for an always-on tray app, Rust backend.
- **Files over sockets** between hook and app: the hook works even when the app is closed,
  and stays a few dozen lines.
- **Registry first, hooks as fallback** (inside `ClaudeCodeSource`): the registry knows sessions
  opened before Cerebro and their names; hooks protect against format changes and add the
  *waiting* status.
- **Source-agnostic domain**: supporting another AI tool means writing its `SessionSource` in
  `cerebro-data` and adding it to `sources` in the injector; the domain doesn't change.
- **Deterministic randomness** everywhere: nothing to store per frame, stable across restarts.
- **The tool's own remote control** over typing into terminals: Claude Code's Remote Control is
  built for it (history, permissions, any network); keystroke injection would be fragile and
  terminal-specific.
- **A web page over a native app**: no store, no signing, one UI build for both. Served over the
  private network (ZeroTier) rather than the internet: nothing exposed publicly, and a key on top
  because a private network can still have other members.
- **Server-sent events** over WebSockets: the phone only listens; SSE reconnects by itself and
  is plain HTTP.
