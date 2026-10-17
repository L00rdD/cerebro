# Cerebro

**Your Claude Code sessions, as X-Men in the X-Mansion. And a hands-on way to learn Rust.**

Cerebro has two goals:

1. **A useful app.** A cross-platform tray app (macOS, Windows, Linux) that shows every running
   Claude Code session as a room of the X-Mansion, seen from above. Glance at it to see which sessions are working, which are
   asleep, and which are waiting for you, then click one to jump to its terminal tab.
2. **A Rust course in a real codebase.** The app is written to be improved by someone learning
   Rust. Twelve exercises, from a first function to cross-platform code, each one shipping a real
   improvement to the app. No toy programs: when an exercise is done, Cerebro uses your code.

## The app

- Each session gets a room and an X-Man; each subagent it spawns adds another one. They wander
  while working, fall asleep when the session is idle, and show **needs you** when Claude is
  waiting for a permission.
- 22 rooms (Danger Room, Blackbird Hangar, Cerebro, Beast's Lab…) and 74 characters, drawn as
  pixel-art sprite sheets in `assets/`.
- Stable picks: a session keeps its room and cast for its whole life, even across restarts.
- Click to focus: selects the exact iTerm2 tab on macOS; activates the hosting app otherwise.
- Cross-platform by design: built with Tauri 2 (Rust backend, Svelte UI) rather than a native
  Swift app, so one codebase targets macOS, Windows and Linux. Platform specifics (process
  inspection, terminal focus) live in adapters behind traits.
- Platform status: macOS is complete; Windows and Linux support is built in exercise 10.

## Learning Rust with Cerebro

The learning path lives in **[EXERCISES.md](EXERCISES.md)**. The code ships with deliberate gaps:
functions that call `todo!("Exercise N: …")`, next to tests marked `#[ignore = "exercise N"]`.
Each exercise is to fill one gap:

1. read the chapters listed for the exercise (mostly the Rust Book);
2. run the ignored tests and watch them fail;
3. write the code until they pass, keep Clippy (pedantic) silent, remove the `#[ignore]`.

| Level | Exercises | What you learn |
|---|---|---|
| 1. Foundations | `humanize_duration`, `Display` for sessions, the `cerebro` CLI | functions, `match`, traits, modules, a binary crate |
| 2. Idiomatic Rust | error handling, iterator statistics, Terminal.app support, borrowing | `Result` and `?`, iterators, trait objects, ownership and lifetimes |
| 3. Systems | file watcher, config file, Windows and Linux, tmux (boss) | threads and channels, serde, conditional compilation |
| Bonus | draw a room | pixel art, no Rust |

The architecture is part of the lesson: a pure domain crate, adapters behind traits, dependency
injection at the edge. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before the first exercise.

## Getting started

Requirements: Rust (stable), Node 20+, and the
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS (macOS 12+,
Windows 10+, or a Linux desktop with WebKitGTK).

```sh
npm install
npm run dev            # build the hook sidecar, run the app with hot reload
npm test               # Rust + UI tests
npm run lint:rust      # Clippy (pedantic, warnings are errors)
npm run check          # Svelte / TypeScript checks
npm run install:macos  # macOS: release build, installed to ~/Applications/Cerebro.app
npm run tauri build    # any OS: release bundle in target/release/bundle
```

The npm scripts run Rust commands through `scripts/with-rust.sh`, which puts a real `cargo` first in `PATH`.

On first launch, click **Install** in the panel banner: it registers `cerebro-hook` in
`~/.claude/settings.json` (a backup is written next to it). Without hooks, sessions still show
up, but subagents and the *waiting* status don't.

## CI and releases

- **Branches**: work happens on `develop`. Feature branches (`features/…`, `fix/…`, `refactor/…`)
  start from `develop` and their pull requests target it (it is the default branch on GitHub).
  `main` only receives `develop` at release time.
- **CI** (`.github/workflows/ci.yml`) runs on every pull request, on `main` and `develop`, and on demand from the
  Actions tab: rustfmt, Clippy, `cargo test` and the UI checks. Work headed for `develop` is checked
  on macOS only; anything headed for `main` (a release) on macOS, Linux and Windows.
- **Release** (`.github/workflows/release.yml`): bump the version in `package.json`,
  `src-tauri/tauri.conf.json` and `Cargo.toml`, then push a tag `vX.Y.Z`. Installers for macOS
  (Apple Silicon and Intel), Linux and Windows are attached to a GitHub release. Run it by hand to
  get a draft release instead. Builds are not code-signed: macOS asks for a right-click > Open.

## Project layout

```
crates/cerebro-protocol   file contract shared by hook and app
crates/cerebro-domain     entities, ports, use cases (pure Rust)
crates/cerebro-data       adapters: files, processes, terminal focus, settings
crates/cerebro-hook       the Claude Code hook binary
crates/cerebro-cli        command line companion (built during the exercises)
src-tauri/                desktop shell: tray, panel, commands, DI
ui/                       Svelte UI: cards and canvas scenes
assets/                   sprites and manifests
docs/ARCHITECTURE.md      how it fits together
EXERCISES.md              learn Rust by improving Cerebro
```

## Data and privacy

Cerebro only reads local files (`~/.claude/sessions`, `~/.cerebro`) and runs local system tools
to inspect processes and focus terminals (`ps`, `open` and `osascript` on macOS). Nothing leaves
the machine.

X-Men characters are Marvel properties; this is a personal fan project, not for distribution.
