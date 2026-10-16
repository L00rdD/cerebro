# Learning Rust with Cerebro

Twelve exercises that make Cerebro better while teaching Rust, from your first function to
cross-platform code. Each one is a real improvement: when you finish, the app (or the CLI)
actually uses your code.

## How to work

- **One branch per exercise**: `git switch -c ex01-humanize-duration`.
- **The tests are the judge.** Most exercises ship with tests marked `#[ignore = "exercise N"]`
  next to a stub that calls `todo!()`. Run them, watch them fail, make them pass, then delete
  the `#[ignore]` line:

  ```sh
  npm run test:rust                                    # everything that should already pass
  sh scripts/with-rust.sh cargo test -p cerebro-domain -- --ignored   # the exercise tests
  sh scripts/with-rust.sh cargo test -p cerebro-domain humanize -- --ignored  # just one group
  ```

- **Keep it clean**: `npm run lint:rust` (Clippy in pedantic mode) must stay silent.
- **Stuck?** Each exercise has hints, hidden behind a click. Try for 20 minutes before opening them.
- Read the compiler errors to the end: Rust's errors usually contain the fix.

> `scripts/with-rust.sh` makes sure the real `cargo` is used (your `~/.local/bin/cargo`
> is a Hermes wrapper). Add `alias rcargo='sh scripts/with-rust.sh cargo'` to your shell to save typing.

## Reading list

- **The Rust Book**: <https://doc.rust-lang.org/book/>. Chapters are given per exercise.
- **Rust by Example**: <https://doc.rust-lang.org/rust-by-example/>. Short runnable snippets.
- **Rustlings**: <https://github.com/rust-lang/rustlings>. Tiny drills, good warm-up before exercise 1.
- **std docs**: `rustup doc --std` opens them offline.

Before starting, read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and run the app once (`npm run dev`).

---

## Level 1: Foundations

### Exercise 1: `humanize_duration`

**Goal**: format an elapsed time as `42s`, `9min`, `2h05`, `3d`.<br>
**Concepts**: functions, integer arithmetic, `if`/`match`, `format!`, `String` vs `&str`, unit tests.<br>
**Read**: Book ch. 3 (*Common Programming Concepts*), ch. 11.1 (*How to Write Tests*).<br>
**Where**: `crates/cerebro-domain/src/time.rs`.

1. Run `cargo test -p cerebro-domain humanize -- --ignored` and read the four failing tests.
2. Replace the `todo!()`.
3. Remove the `#[ignore]` attributes.

**Done when** the 4 tests pass. The CLI (exercise 3) will use your function; the UI has a
TypeScript twin in `ui/src/lib/format.ts`. Compare both versions.

<details><summary>Hints</summary>

- Convert to seconds first: `let s = elapsed_ms / 1000;`.
- `match s { 0..=59 => …, 60..=3599 => …, _ => … }` reads nicely.
- Zero padding: `format!("{h}h{m:02}")`.
</details>

### Exercise 2: `Display` for sessions

**Goal**: `session.to_string()` gives `personal-f4  busy  2 agents  ~/personal`.<br>
**Concepts**: traits, `impl Trait for Type`, `std::fmt::Formatter`, `write!`, pattern matching on enums, pluralization.<br>
**Read**: Book ch. 6 (*Enums and Pattern Matching*), ch. 10.2 (*Traits*).<br>
**Where**: `crates/cerebro-domain/src/entities/session.rs` (bottom of the file).

`SessionStatus` already has `as_str()`. Use it rather than duplicating the `match`.

<details><summary>Hints</summary>

- `write!(f, "{}", self.as_str())` returns the `fmt::Result` you need.
- Inside `Display for Session`, `self.status` already implements `Display` (once you've written it), so `{}` works on it.
- `let agents = match self.agents.len() { 1 => "1 agent".to_string(), n => format!("{n} agents") };`
</details>

### Exercise 3: the `cerebro` CLI

**Goal**: `cargo run -p cerebro-cli -- list` prints the live sessions; `stats` and `hooks` follow.<br>
**Concepts**: binary crates, workspaces, reusing library crates, `Result` in `main`, `ExitCode`, `Arc<dyn Trait>`.<br>
**Read**: Book ch. 7 (*Packages, Crates, and Modules*), ch. 12 (*An I/O Project*).<br>
**Where**: `crates/cerebro-cli/src/main.rs`.

1. Build an `ObserveSessions` exactly like `src-tauri/src/injector.rs` does. Wonder why you can
   copy it without touching Tauri: that's the point of the architecture.
2. `list`: one line per scene using exercise 2, plus `humanize_duration` for the status age.
3. `hooks`: print `installed` / `missing` / `outdated` with `ManageHooks::status`.
4. `stats`: after exercise 5.

**Done when** `cargo run -p cerebro-cli -- list` shows the same sessions as the panel.

<details><summary>Hints</summary>

- `fn main() -> Result<(), Box<dyn std::error::Error>>` lets you use `?` everywhere.
- The hook binary path does not matter for `status()`; reuse the injector's logic or pass a dummy path.
- Bonus: parse arguments with the `clap` crate (`cargo add clap --features derive -p cerebro-cli`).
</details>

---

## Level 2: Idiomatic Rust

### Exercise 4: stop swallowing errors

**Goal**: the registry scan reports which files were unreadable or malformed instead of ignoring them.<br>
**Concepts**: `Result`, the `?` operator, custom error enums, `thiserror`, `map_err`, partitioning results.<br>
**Read**: Book ch. 9 (*Error Handling*); the `thiserror` README.<br>
**Where**: `crates/cerebro-data/src/datasources/claude_registry.rs` (`scan`, `RegistryScan`, `RegistryProblem`).

1. Implement `scan()`. Files that are not `.json` are *not* problems.
2. Derive `thiserror::Error` on `RegistryProblem` with readable messages (add `thiserror` to the crate).
3. Make `observe()` call `scan()` and log the problems with `eprintln!`.
4. Stretch: surface the problem count in the UI. Add a field to `DataMode` or `Snapshot`,
   map it in `src-tauri/src/dto.rs` and `ui/src/lib/api/types.ts`, show it in `Banners.svelte`.

**Done when** `scan_reports_malformed_files` passes.

<details><summary>Hints</summary>

- `Iterator::partition` or a `for` loop with two `Vec`s both work. Start with the loop, then try `partition`.
- `fs::read(&path).map_err(|e| RegistryProblem::Unreadable(e.to_string()))?` inside a closure returning `Result`.
</details>

### Exercise 5: statistics with iterators

**Goal**: implement `compute_stats` (agents by type, busiest session, longest running agent).<br>
**Concepts**: iterators and adapters (`flat_map`, `fold`, `max_by_key`, `min_by_key`), closures, `BTreeMap`, `Option` combinators.<br>
**Read**: Book ch. 13 (*Iterators and Closures*), ch. 8.3 (*Hash Maps*).<br>
**Where**: `crates/cerebro-domain/src/entities/stats.rs`.

Rule of the game: no `for` loop and no `let mut`. Then wire it into `cerebro stats`.

**Done when** the 3 tests pass and `cerebro stats` prints them.

<details><summary>Hints</summary>

- Counting: `.fold(BTreeMap::new(), |mut map, kind| { *map.entry(kind).or_insert(0) += 1; map })`. The `mut` is on the closure parameter, which is allowed.
- `max_by_key` returns the *last* maximum; for "oldest wins a tie", compare on `(agents.len(), Reverse(started_at))`.
- `Timestamp::since` already saturates for you.
</details>

### Exercise 6: Terminal.app support

**Goal**: clicking a card focuses the right tab in macOS Terminal, not only in iTerm2.<br>
**Concepts**: implementing a trait for a new type, trait objects (`Box<dyn TabSelector>`), testing with fakes, raw strings.<br>
**Read**: Book ch. 17.2 (*Trait Objects*), ch. 19.3 for raw strings (`r#"…"#`).<br>
**Where**: `crates/cerebro-data/src/strategies/focus/terminal_app.rs`, then register it in `MacFocuser::new` (`focus/mod.rs`).

Read `iterm.rs` first: your selector is its sibling. The tests use `ScriptedRunner`, a fake
`CommandRunner`, so they never launch AppleScript.

**Done when** the 3 tests pass and a real Terminal.app tab gets selected when you click its card.

<details><summary>Hints</summary>

```applescript
tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            if tty of t is "/dev/ttys007" then
                set selected of t to true
                set index of w to 1
                activate
                return "found"
            end if
        end repeat
    end repeat
end tell
return "missing"
```
</details>

### Exercise 7: borrowing instead of copying

**Goal**: parse `ps` output without allocating, returning slices of the input.<br>
**Concepts**: references, lifetimes (`PsLine<'a>`), `&str` slicing, `split_whitespace`, `Iterator::eq`.<br>
**Read**: Book ch. 4 (*Understanding Ownership*), ch. 10.3 (*Validating References with Lifetimes*).<br>
**Where**: `crates/cerebro-data/src/services/process_inspector.rs` (`PsLine`, `parse_ps_line_borrowed`).

1. Implement `parse_ps_line_borrowed` and `PsLine::started_at`.
2. Switch `PsProcessInspector` to it. `ProcessInfo` stores a `String`, so where must the single
   allocation now happen?
3. Delete the old `parse_ps_line`. The compiler lists every place to update.

**Done when** `borrowed_parse_points_into_the_input` passes (it checks that no copy was made).

<details><summary>Hints</summary>

- `line.trim_start().split_once(char::is_whitespace)` gives you two `&str` pointing into `line`.
- `a.split_whitespace().eq(b.split_whitespace())` compares word by word without building a `String`.
- Try returning a `PsLine` from a function that owns its `String`: read the error, that's the lesson.
</details>

---

## Level 3: Systems programming

### Exercise 8: react instantly with a file watcher

**Goal**: update the panel as soon as a session or agent file changes, instead of polling every 1-2 s.<br>
**Concepts**: threads, channels (`std::sync::mpsc`), `Arc<Mutex<_>>`, debouncing, external crates (`notify`).<br>
**Read**: Book ch. 16 (*Fearless Concurrency*); the `notify` crate docs.<br>
**Where**: `src-tauri/src/poller.rs` (and maybe a new `watcher.rs`).

1. Watch `~/.claude/sessions` and `~/.cerebro` recursively.
2. Debounce: many events in 150 ms should trigger a single `refresh`.
3. Keep a slow safety poll (10 s), because processes can die without touching any file.

**Done when** starting a subagent shows its character in under half a second, and idle CPU does not increase.

<details><summary>Hints</summary>

- `notify::recommended_watcher(tx)` sends events into an `mpsc::Sender`.
- Debounce loop: block on `rx.recv()`, then drain with `rx.recv_timeout(150ms)` until it times out, then refresh once.
</details>

### Exercise 9: a configuration file

**Goal**: `~/.cerebro/config.toml` with `favorite_characters`, `banned_rooms` and `refresh_seconds`.<br>
**Concepts**: `serde` derive, `#[serde(default)]`, `Default`, the `toml` crate, validating input, passing config through layers.<br>
**Read**: Book ch. 5 (*Structs*); serde.rs *Attributes* page.<br>
**Where**: a new `Config` in `cerebro-protocol` or `cerebro-data`, consumed by `strategies/casting.rs` and the poller.

Design question first: should casting know about files? (No. Pass it plain data.)
Write the tests before the code: a missing file means defaults, and an unknown character id is ignored.

### Exercise 10: Windows and Linux

**Goal**: Cerebro works outside macOS.<br>
**Concepts**: `#[cfg(target_os = "…")]`, platform crates, conditional dependencies in `Cargo.toml`.<br>
**Read**: *The Cargo Book*, "Platform specific dependencies"; the `sysinfo` crate.<br>
**Where**: `services/process_inspector.rs` (the `cfg!(windows)` branch), `strategies/focus/mod.rs` (`platform_focuser`).

1. Real process start times on Windows (`sysinfo`), and check their format against what Claude Code writes there.
2. A Linux focuser using `wmctrl -lp` (find the window of the terminal pid) then `wmctrl -ia`.
3. A Windows focuser bringing the terminal window to the front.
4. Build the installers in CI (GitHub Actions with the `tauri-action`).

### Exercise 11 (boss): tmux

**Goal**: clicking a session running inside tmux selects the right tmux window and pane, then focuses the terminal.<br>
**Concepts**: everything above, plus parsing command output robustly.<br>
**Hint**: `tmux list-panes -a -F '#{pane_tty} #{session_name}:#{window_index}.#{pane_index}'`, then
`tmux select-window -t …` / `select-pane -t …`; the terminal hosting tmux is the client's: `tmux list-clients -F '#{client_tty} #{client_pid}'`.

### Exercise 12 (no Rust): draw a room

**Goal**: add a new room to the Institute without writing code.
Draw a 96×60 PNG in Aseprite or Piskel (a strip of frames if animated), add an entry to
`assets/rooms/rooms.json` with its `walkArea`, run the app. Then ask yourself which Rust test
guarantees the manifest stays valid (`shipped_manifests_are_valid`).

---

## After the exercises

- Read the code of `strategies/casting.rs` and `usecases/observe_sessions.rs` again. You should now understand every line.
- Pick an item from the review report (`docs/` or the HTML report) and fix it on your own.
- *Rust for Rustaceans* (Jon Gjengset) is the natural next book.
