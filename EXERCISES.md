# Learning Rust with Cerebro

Twelve exercises that make Cerebro better while teaching Rust, from your first function to
cross-platform code. Each one is a real improvement: when you finish, the app (or the CLI)
actually uses your code.

## How to work

Every exercise follows the same loop. Exercise 2 below walks through it command by command; read
it once even if you start elsewhere.

### 1. Start a branch

One branch per exercise, from `develop`. Each exercise gives its branch name: `features/` for
something new, `fix/` for a bug, `refactor/` when behavior stays the same.

```sh
git switch develop && git pull
git switch -c features/ex02_session_display
```

### 2. Find the stub and its tests

The **Where** line of the exercise gives the file. Inside it you'll find:

- the code to write, marked `EXERCISE N` and ending in `todo!(…)`. `todo!()` compiles but
  panics when it runs: it's a placeholder;
- at the bottom, a `mod tests` block whose tests carry `#[ignore = "exercise N"]`. Ignored tests
  are skipped by a normal `cargo test`, so the project stays green until you start.

### 3. Run the exercise's tests and watch them fail

All commands run from the repository root. Each exercise gives its exact **Test** command, e.g.:

```sh
cargo test -p cerebro-domain session::tests -- --ignored
```

How to read that line:

| Part | Meaning |
|---|---|
| `-p cerebro-domain` | only this crate (package) of the workspace, so it compiles faster |
| `session::tests` | only the tests whose path contains this text (a filter, not an exact name) |
| `--` | what follows goes to the test runner, not to Cargo |
| `--ignored` | run **only** the `#[ignore]` tests, i.e. the exercise ones |

The first run says `FAILED` with `not yet implemented: Exercise 2…`: that's the `todo!()`
panicking. That's normal, it's your starting point.

### 4. Write the code until the tests pass

Replace the `todo!()` (and the `let _ = f;` line next to it, which only silences a warning while
`f` is unused). Run the same command after each change. When it ends with
`test result: ok. 2 passed`, you're done.

- Read compiler errors to the end: the `help:` line often contains the fix.
- To see what your code prints, add `println!("{line}");` in a test and run with
  `-- --ignored --nocapture` (the test runner hides output otherwise).
- **Stuck?** Each exercise has hints, hidden behind a click. Try for 20 minutes before opening them.

### 5. Remove the `#[ignore]` lines and check everything

Your tests are now regular tests. Delete the `#[ignore = "exercise N"]` lines, then run what CI
runs:

```sh
npm run test:rust   # every test of the workspace (cargo test --workspace)
npm run lint:rust   # Clippy in pedantic mode: must print no warning
cargo fmt --all     # format the code the way CI expects
```

### 6. Commit, push, open the pull request

```sh
git add -A
git commit -m "Exercise 2: Display for sessions"
git push -u origin HEAD
gh pr create --fill
```

The PR targets `develop`. GitHub Actions runs the tests on macOS, Linux and Windows; merge
once everything is green.

> If `cargo` ever turns out not to be the real Cargo (e.g. a wrapper earlier in `PATH`),
> prefix the commands with `sh scripts/with-rust.sh`: it puts the rustup toolchain first.

## Reading list

- **The Rust Book**: <https://doc.rust-lang.org/book/>. Chapters are given per exercise.
- **Rust by Example**: <https://doc.rust-lang.org/rust-by-example/>. Short runnable snippets.
- **Rustlings**: <https://github.com/rust-lang/rustlings>. Tiny drills, good warm-up before exercise 1.
- **std docs**: `rustup doc --std` opens them offline.

Before starting, read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and run the app once (`npm run dev`).

---

## Level 1: Foundations

### Exercise 1: `humanize_duration`

**Branch**: `features/ex01_humanize_duration`.<br>
**Goal**: format an elapsed time as `42s`, `9min`, `2h05`, `3d`.<br>
**Concepts**: functions, integer arithmetic, `if`/`match`, `format!`, `String` vs `&str`, unit tests.<br>
**Read**: Book [ch. 3 (*Common Programming Concepts*)](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html), [ch. 11.1 (*How to Write Tests*)](https://doc.rust-lang.org/book/ch11-01-writing-tests.html).<br>
**Where**: `crates/cerebro-domain/src/time.rs`.<br>
**Test**: `cargo test -p cerebro-domain humanize -- --ignored` (4 tests; once done, drop `--ignored`).

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

**Branch**: `features/ex02_session_display`.<br>
**Goal**: `session.to_string()` gives `personal-f4  busy  2 agents  /Users/me/personal`.<br>
**Concepts**: traits, `impl Trait for Type`, `std::fmt::Formatter`, `write!`, pattern matching on enums, pluralization.<br>
**Read**: Book [ch. 6 (*Enums and Pattern Matching*)](https://doc.rust-lang.org/book/ch06-00-enums.html), [ch. 10.2 (*Traits*)](https://doc.rust-lang.org/book/ch10-02-traits.html); [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html).<br>
**Where**: `crates/cerebro-domain/src/entities/session.rs`, the two `impl std::fmt::Display` blocks near the bottom.<br>
**Test**: `cargo test -p cerebro-domain session::tests -- --ignored` (2 tests).

**Why.** A *trait* is a set of methods a type promises to have. `Display` has just one, `fmt`,
which says how to write the value as text. Implement it and you get, for free, `to_string()`,
`println!("{session}")` and `format!("{session}")`. The CLI of exercise 3 prints sessions this way.

**What to write.** Two implementations:

1. `Display for SessionStatus`: `busy`, `idle`, `waiting`, or the raw text for an unknown status
   (`Other("compacting")` gives `compacting`). The method `as_str()` just above already does the
   `match`: reuse it instead of writing a second one.
2. `Display for Session`: the name, the status, the number of agents and the folder, separated
   by **two spaces**:

   ```text
   personal-f4  busy  2 agents  /Users/me/personal
   personal-f4  idle  1 agent  /Users/me/personal      ← singular for 1
   personal-f4  waiting  0 agents  /Users/me/personal
   ```

**How `fmt` works.** You don't return a `String`: you *write* into `f` (the `Formatter`) with the
`write!` macro, which takes the same arguments as `format!` and returns the `std::fmt::Result`
that `fmt` must return:

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "hello {}", 42)
}
```

**Steps.**

1. `git switch -c features/ex02_session_display` (from an up-to-date `develop`).
2. Run the **Test** command: 2 tests fail with `not yet implemented`.
3. Write `Display for SessionStatus`. `status_display` passes; `session_display_counts_agents`
   still fails.
4. Write `Display for Session`. `cwd` is a `PathBuf`, which does *not* implement `Display`
   (a path isn't always valid text): call `.display()` on it.
5. Both tests pass: remove the two `#[ignore = "exercise 2"]` lines and finish with steps 5 and 6
   of [How to work](#how-to-work).

**Done when** the 2 tests pass without `--ignored`, and Clippy is silent.<br>
**Bonus**: show `~/personal` instead of `/Users/me/personal`. Where should that live: in the
domain (which must not read the environment) or in the CLI? The UI does it in `shortenHome`
(`ui/src/lib/format.ts`).

<details><summary>Hints</summary>

- `write!(f, "{}", self.as_str())` is the whole body of the first `fmt`.
- Inside `Display for Session`, `self.status` implements `Display` once step 3 is done, so `{}` works on it.
- `let agents = match self.agents.len() { 1 => "1 agent".to_string(), n => format!("{n} agents") };`
- Then: `write!(f, "{}  {}  {agents}  {}", self.name, self.status, self.cwd.display())`.
</details>

### Exercise 3: the `cerebro` CLI

**Branch**: `features/ex03_cli`.<br>
**Goal**: `cargo run -p cerebro-cli -- list` prints the live sessions; `stats` and `hooks` follow.<br>
**Concepts**: binary crates, workspaces, reusing library crates, `Result` in `main`, `ExitCode`, `Arc<dyn Trait>`.<br>
**Read**: Book [ch. 7 (*Packages, Crates, and Modules*)](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html), [ch. 12 (*An I/O Project*)](https://doc.rust-lang.org/book/ch12-00-an-io-project.html).<br>
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

**Branch**: `fix/ex04_registry_scan_errors`.<br>
**Goal**: the registry scan reports which files were unreadable or malformed instead of ignoring them.<br>
**Concepts**: `Result`, the `?` operator, custom error enums, `thiserror`, `map_err`, partitioning results.<br>
**Read**: Book [ch. 9 (*Error Handling*)](https://doc.rust-lang.org/book/ch09-00-error-handling.html); the [`thiserror` docs](https://docs.rs/thiserror).<br>
**Where**: `crates/cerebro-data/src/datasources/claude_registry.rs` (`scan`, `RegistryScan`, `RegistryProblem`).<br>
**Test**: `cargo test -p cerebro-data scan_reports_malformed_files -- --ignored`.

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

**Branch**: `features/ex05_session_stats`.<br>
**Goal**: implement `compute_stats` (agents by type, busiest session, longest running agent).<br>
**Concepts**: iterators and adapters (`flat_map`, `fold`, `max_by_key`, `min_by_key`), closures, `BTreeMap`, `Option` combinators.<br>
**Read**: Book [ch. 13 (*Iterators and Closures*)](https://doc.rust-lang.org/book/ch13-00-functional-features.html), [ch. 8.3 (*Hash Maps*)](https://doc.rust-lang.org/book/ch08-03-hash-maps.html).<br>
**Where**: `crates/cerebro-domain/src/entities/stats.rs`.<br>
**Test**: `cargo test -p cerebro-domain stats -- --ignored` (3 tests).

Rule of the game: no `for` loop and no `let mut`. Then wire it into `cerebro stats`.

**Done when** the 3 tests pass and `cerebro stats` prints them.

<details><summary>Hints</summary>

- Counting: `.fold(BTreeMap::new(), |mut map, kind| { *map.entry(kind).or_insert(0) += 1; map })`. The `mut` is on the closure parameter, which is allowed.
- `max_by_key` returns the *last* maximum; for "oldest wins a tie", compare on `(agents.len(), Reverse(started_at))`.
- `Timestamp::since` already saturates for you.
</details>

### Exercise 6: Terminal.app support

**Branch**: `features/ex06_terminal_app_focus`.<br>
**Goal**: clicking a card focuses the right tab in macOS Terminal, not only in iTerm2.<br>
**Concepts**: implementing a trait for a new type, trait objects (`Box<dyn TabSelector>`), testing with fakes, raw strings.<br>
**Read**: Book [ch. 18.2 (*Trait Objects*)](https://doc.rust-lang.org/book/ch18-02-trait-objects.html); the Reference, [*Raw string literals*](https://doc.rust-lang.org/reference/tokens.html#raw-string-literals), for `r#"…"#`.<br>
**Where**: `crates/cerebro-data/src/strategies/focus/terminal_app.rs`, then register it in `MacFocuser::new` (`focus/mod.rs`).<br>
**Test**: `cargo test -p cerebro-data terminal_app -- --ignored` (3 tests).

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

**Branch**: `refactor/ex07_borrowed_ps_parsing`.<br>
**Goal**: parse `ps` output without allocating, returning slices of the input.<br>
**Concepts**: references, lifetimes (`PsLine<'a>`), `&str` slicing, `split_whitespace`, `Iterator::eq`.<br>
**Read**: Book [ch. 4 (*Understanding Ownership*)](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html), [ch. 10.3 (*Validating References with Lifetimes*)](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).<br>
**Where**: `crates/cerebro-data/src/services/process_inspector.rs` (`PsLine`, `parse_ps_line_borrowed`).<br>
**Test**: `cargo test -p cerebro-data borrowed_parse -- --ignored`.

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

**Branch**: `refactor/ex08_file_watcher`.<br>
**Goal**: update the panel as soon as a session or agent file changes, instead of polling every 1-2 s.<br>
**Concepts**: threads, channels (`std::sync::mpsc`), `Arc<Mutex<_>>`, debouncing, external crates (`notify`).<br>
**Read**: Book [ch. 16 (*Fearless Concurrency*)](https://doc.rust-lang.org/book/ch16-00-concurrency.html); the [`notify` crate docs](https://docs.rs/notify).<br>
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

**Branch**: `features/ex09_config_file`.<br>
**Goal**: `~/.cerebro/config.toml` with `favorite_characters`, `banned_rooms` and `refresh_seconds`.<br>
**Concepts**: `serde` derive, `#[serde(default)]`, `Default`, the `toml` crate, validating input, passing config through layers.<br>
**Read**: Book [ch. 5 (*Structs*)](https://doc.rust-lang.org/book/ch05-00-structs.html); serde.rs [*Attributes*](https://serde.rs/attributes.html) page.<br>
**Where**: a new `Config` in `cerebro-protocol` or `cerebro-data`, consumed by `strategies/casting.rs` and the poller.

Design question first: should casting know about files? (No. Pass it plain data.)
Write the tests before the code: a missing file means defaults, and an unknown character id is ignored.

### Exercise 10: Windows and Linux

**Branch**: `features/ex10_windows_linux`.<br>
**Goal**: Cerebro works outside macOS.<br>
**Concepts**: `#[cfg(target_os = "…")]`, platform crates, conditional dependencies in `Cargo.toml`.<br>
**Read**: *The Cargo Book*, ["Platform specific dependencies"](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies); the [`sysinfo` crate](https://docs.rs/sysinfo).<br>
**Where**: `services/process_inspector.rs` (the `cfg!(windows)` branch), `strategies/focus/mod.rs` (`platform_focuser`).

1. Real process start times on Windows (`sysinfo`), and check their format against what Claude Code writes there.
2. A Linux focuser using `wmctrl -lp` (find the window of the terminal pid) then `wmctrl -ia`.
3. A Windows focuser bringing the terminal window to the front.
4. Build the installers in CI (GitHub Actions with the `tauri-action`).

### Exercise 11 (boss): tmux

**Branch**: `features/ex11_tmux_focus`.<br>
**Goal**: clicking a session running inside tmux selects the right tmux window and pane, then focuses the terminal.<br>
**Concepts**: everything above, plus parsing command output robustly.<br>
**Read**: the [tmux manual](https://man7.org/linux/man-pages/man1/tmux.1.html) (`FORMATS` section).<br>
**Hint**: `tmux list-panes -a -F '#{pane_tty} #{session_name}:#{window_index}.#{pane_index}'`, then
`tmux select-window -t …` / `select-pane -t …`; the terminal hosting tmux is the client's: `tmux list-clients -F '#{client_tty} #{client_pid}'`.

### Exercise 12 (no Rust): draw a room

**Branch**: `features/ex12_new_room`.<br>
**Goal**: add a new room to the Institute without writing code.
Draw a 96×60 PNG in Aseprite or Piskel (a strip of frames if animated), add an entry to
`assets/rooms/rooms.json` with its `walkArea`, run the app. Then ask yourself which Rust test
guarantees the manifest stays valid (`shipped_manifests_are_valid`).

---

## After the exercises

- Read the code of `strategies/casting.rs` and `usecases/observe_sessions.rs` again. You should now understand every line.
- Pick an item from the review report (`docs/` or the HTML report) and fix it on your own.
- *Rust for Rustaceans* (Jon Gjengset) is the natural next book.
