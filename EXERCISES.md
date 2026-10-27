# Learning Rust with Cerebro

Thirteen exercises that make Cerebro better while teaching Rust, from your first function to
cross-platform and async code. Each one is a real improvement: when you finish, the app (or the CLI)
actually uses your code.

Every exercise has the same parts:

- a header: branch, goal, concepts, reading, the files to touch (**Where**) and the test command;
- **Why**: what the feature is for, so you know what "correct" means;
- **What to build**: the exact expected behavior (output, rules, edge cases). If something you need
  is not written here, it's a bug in this document: open an issue;
- **Done when**: the checklist to tick before opening the PR;
- **Hints**, hidden behind a click. Try for 20 minutes before opening them.

## How to work

Every exercise follows the same loop.

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

Some exercises (3, 8 to 13) have no prepared tests: their **Done when** says how to check them.

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

The first run says `FAILED` with `not yet implemented: Exercise N…`: that's the `todo!()`
panicking. That's normal, it's your starting point.

### 4. Write the code until the tests pass

Replace the `todo!()`, plus any `let _ = …;` line next to it (those only silence an "unused
variable" warning while the stub is empty). Run the same command after each change, until it ends
with `test result: ok`.

- Read compiler errors to the end: the `help:` line often contains the fix.
- When the code does not compile, the types shown by your editor are guesses. Trust `cargo build`.
- To see what your code prints, add `println!("{line}");` in a test and run with
  `-- --ignored --nocapture` (the test runner hides output otherwise).

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
- **Tokio tutorial**: <https://tokio.rs/tokio/tutorial>. Async Rust, for exercise 13.

Before starting, read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and run the app once (`npm run dev`).

## Vocabulary used below

| Term | Meaning |
|---|---|
| **session** | one running Claude Code conversation (`Session` in `cerebro-domain/src/entities/session.rs`) |
| **agent** | a subagent launched by a session (`Agent`, same file). Only known through the hooks |
| **scene** | a session plus its *cast*: the room and characters drawn for it (`Scene`) |
| **snapshot** | everything observed at one instant: the scenes, warnings, and `taken_at`, the instant of the observation (`Snapshot`) |
| **`Timestamp`** | an *instant*: milliseconds since 1970-01-01 (`time.rs`). Not a duration. `later.since(earlier)` gives the duration between two instants, in ms |
| **registry** | Claude Code's own files, `~/.claude/sessions/<pid>.json`, one per live session |
| **hooks** | entries in `~/.claude/settings.json` that make Claude Code run `cerebro-hook` on every event (session started, prompt sent, subagent started…). The hook writes what it sees under `~/.cerebro/` |
| **use case** | a struct in `cerebro-domain/src/usecases/` doing one job (`ObserveSessions`, `ManageHooks`, `FocusSession`). Its dependencies are private: use its public methods only |
| **injector** | the code that builds the use cases with their real adapters (`src-tauri/src/injector.rs` for the app) |

---

## Level 1: Foundations

### Exercise 1: `humanize_duration`

**Branch**: `features/ex01_humanize_duration`.<br>
**Goal**: format an elapsed time as `42s`, `9min`, `2h05`, `3d`.<br>
**Concepts**: functions, integer arithmetic, `if`/`match`, `format!`, `String` vs `&str`, unit tests.<br>
**Read**: Book [ch. 3 (*Common Programming Concepts*)](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html), [ch. 11.1 (*How to Write Tests*)](https://doc.rust-lang.org/book/ch11-01-writing-tests.html).<br>
**Where**: `crates/cerebro-domain/src/time.rs`, function `humanize_duration`.<br>
**Test**: `cargo test -p cerebro-domain humanize -- --ignored` (4 tests).

**Why.** Session cards and the CLI show how long a session has been in its status. A raw number of
milliseconds is unreadable; `2h05` is not.

**What to build.** `humanize_duration(elapsed_ms: u64) -> String`. The input is a **duration** in
milliseconds (not an instant). Rules, applied to the duration rounded *down* to whole seconds:

| Duration | Output | Example |
|---|---|---|
| under 1 minute | seconds + `s` | `42_999` ms → `42s`, `0` → `0s` |
| under 1 hour | minutes + `min` | `60_000` → `1min`, `3_599_000` → `59min` |
| under 1 day | hours + `h` + minutes on 2 digits | `7_500_000` → `2h05` |
| 1 day or more | days + `d` | `3 * 86_400_000` → `3d` |

Always round down: `59.9 s` is `59s`, never `1min`.

**Done when**

- [ ] the 4 tests pass without `--ignored`;
- [ ] Clippy is silent.

The UI has a TypeScript twin in `ui/src/lib/format.ts` (`humanizeDuration`): compare both versions.

<details><summary>Hints</summary>

- Convert to seconds first, then work only with seconds.
- A `match` on ranges (`0..=59 => …`) reads well for the four cases.
- Zero padding in `format!` is written `{value:02}`.
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
`println!("{session}")` and `format!("{session}")`.

**What to build.** Two implementations:

1. `Display for SessionStatus`: `busy`, `idle`, `waiting`, or the raw text for an unknown status
   (`Other("compacting")` gives `compacting`). The method `as_str()` just above already does the
   `match`: reuse it.
2. `Display for Session`: the name, the status, the number of agents and the folder, separated
   by **two spaces**, with "agent" in the singular for exactly 1:

   ```text
   personal-f4  busy  2 agents  /Users/me/personal
   personal-f4  idle  1 agent  /Users/me/personal
   personal-f4  waiting  0 agents  /Users/me/personal
   ```

**How `fmt` works.** You don't return a `String`: you *write* into `output` (the `Formatter`) with the
`write!` macro, which takes the same arguments as `format!` and returns the `std::fmt::Result`
that `fmt` must return:

```rust
fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(output, "hello {}", 42)
}
```

**Good to know: width and alignment.** `format!("{:<10}", value)` pads `value` to 10 characters,
but only if its `Display` cooperates. `write!` ignores the requested width; `output.pad(text)`
honors it. Use `pad` for `SessionStatus` (a single word that callers may want to align). For
`Session` it does not matter: it's a whole line.

`cwd` is a `PathBuf`, which does *not* implement `Display` (a path isn't always valid text):
call `.display()` on it.

**Done when**

- [ ] `status_display` and `session_display_counts_agents` pass without `--ignored`;
- [ ] Clippy is silent.

**Bonus**: show `~/personal` instead of `/Users/me/personal`. Should that live in the domain
(which must not read the environment) or in the CLI? The UI does it in `shortenHome`
(`ui/src/lib/format.ts`).

<details><summary>Hints</summary>

- The first `fmt` is a one-liner around `self.as_str()`.
- Inside `Display for Session`, `self.status` implements `Display` once the first impl is done,
  so `{}` works on it.
- Compute the "agent"/"agents" word before the `write!`, with an `if` or a `match` on
  `self.agents.len()`.
</details>

### Exercise 3: the `cerebro` CLI

**Branch**: `features/ex03_cli`.<br>
**Goal**: `cerebro list` and `cerebro hooks` work from the terminal (`stats` comes in exercise 5).<br>
**Concepts**: binary crates, workspaces, reusing library crates, modules, `ExitCode`, `let … else`, `match` on `Result`, format width.<br>
**Read**: Book [ch. 7 (*Packages, Crates, and Modules*)](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html), [ch. 12 (*An I/O Project*)](https://doc.rust-lang.org/book/ch12-00-an-io-project.html); [`std::fmt` (*Width*, *Fill / Alignment*)](https://doc.rust-lang.org/std/fmt/#width).<br>
**Where**: `crates/cerebro-cli/src/` (`main.rs`, plus any module you add).<br>
**Test**: none prepared; check by running `cargo run -p cerebro-cli -- <command>`.

**Why.** The CLI is a second entry point on the same core as the app. If the architecture is right,
it reuses the domain and the adapters without a single line of Tauri. It's also handy: a quick
`cerebro list` in a terminal, or a script that checks the hooks.

**What to build.**

**1. The wiring.** The CLI needs two use cases: `ObserveSessions` (for `list`) and `ManageHooks`
(for `hooks`). Build them with the same adapters as `src-tauri/src/injector.rs`. Put that wiring
wherever you find it cleanest: in `main.rs`, or in a dedicated module (e.g. a CLI `injector.rs`
with its own `Container`). The CLI does not need `FocusSession`. If the wiring fails, print an
error on stderr and exit with code 1.

**2. `cerebro list`.** One line per live session, in the order of `snapshot.scenes`. Each line shows,
in aligned columns:

| Column | Source | Example |
|---|---|---|
| name | `session.name` | `cerebro-6d` |
| status | `session.status` | `busy` |
| agents | `session.agents.len()`, singular for 1 | `2 agents` |
| status age | `humanize_duration(snapshot.taken_at.since(session.status_since))` | `9min` |
| folder | `session.cwd` | `/Users/me/personal/cerebro` |

```text
d-mediaserver-10  idle  0 agents  24min  /Users/me/personal/d-mediaserver
cerebro-6d        busy  2 agents    19s  /Users/me/personal/cerebro
```

- The *status age* is how long the session has been in its current status: from
  `status_since` (when the status last changed) to `taken_at` (when the snapshot was taken). Do not
  use `started_at` (launch time), and do not read a clock yourself: `taken_at` already is "now",
  the same for every line.
- Columns are aligned: each one is as wide as its longest value. The last column needs no padding.
- You may reuse the `Display` of exercise 2 or format the fields one by one. Note that a
  `Display` built with `write!` ignores widths, so aligning *inside* it is not possible.
- No live session: print `no live session` and exit with code 0.
- If `snapshot.warnings` is not empty (e.g. the registry is unreadable), print each warning on
  **stderr**, then the list as usual.

**3. `cerebro hooks`.** Prints whether Cerebro's hooks are installed in Claude Code.

Without hooks, Cerebro only sees Claude Code's registry: it still lists sessions, but **no
subagents** (agent counts stay at 0) and **no `waiting` status**. So when the panel looks wrong,
this is the first thing to check. The app shows the same information as a banner
(`ui/src/components/Banners.svelte`).

`ManageHooks::status()` returns `Result<HookSetup, DomainError>`. Print exactly one line on
stdout, then exit:

| `HookSetup` | Output | Exit code |
|---|---|---|
| `Installed` | `hooks: installed` | 0 |
| `Missing` | `hooks: missing (subagents and the waiting status are not tracked)` | 1 |
| `Outdated` | `hooks: outdated (they point to another cerebro-hook or miss some events)` | 1 |
| `Err(error)` | nothing on stdout; `cannot read the hooks: {error}` on stderr | 1 |

**Careful with the hook binary path.** `ManageHooks` is built with the path of the `cerebro-hook`
binary it expects, and `status()` compares it with the command written in
`~/.claude/settings.json`. The app's `hook_binary_path()` looks next to the running executable.
Copied as is into the CLI, it expects `target/debug/cerebro-hook`, while hooks installed by the
app point to `…/Cerebro.app/Contents/MacOS/cerebro-hook`: the CLI would say `outdated` when the
app says everything is fine. Decide what the CLI should compare against and write it down in a
comment. Any reasoned choice is accepted; reporting a false `outdated` is not.

**4. `cerebro stats`.** Leave it as is (`todo!()` or a "not implemented yet" message):
exercise 5 implements it.

**Exit codes.** 0 on success, 1 on a runtime error (or hooks not installed), 2 on a bad command
line (already done by the stub).

**Done when**

- [ ] `cargo run -p cerebro-cli -- list` shows the same sessions as the panel, with the same agent
  counts and coherent status ages;
- [ ] `cargo run -p cerebro-cli -- hooks` agrees with the app (no banner in the app ⇔ `installed`);
- [ ] `echo $?` after each command gives the codes above;
- [ ] Clippy is silent (`npm run lint:rust`).

<details><summary>Hints</summary>

- `?` only works in a function returning `Result` (or `Option`). Either make `main` return
  `Result<(), Box<dyn std::error::Error>>`, or keep `ExitCode` and handle the error with
  `let Ok(value) = … else { …; return ExitCode::FAILURE; };` or a `match`.
- Format width can come from a variable: `{value:<width$}`.
- Aligning columns takes two passes over the rows: one to measure, one to print.
- For widths, count characters with `.chars().count()`, not bytes with `.len()`.
- Bonus: parse arguments with the `clap` crate (`cargo add clap --features derive -p cerebro-cli`).
</details>

---

## Level 2: Idiomatic Rust

### Exercise 4: stop swallowing errors

**Branch**: `fix/ex04_registry_scan_errors`.<br>
**Goal**: the registry scan reports which files were unreadable or malformed instead of silently ignoring them.<br>
**Concepts**: `Result`, the `?` operator, custom error enums, `thiserror`, `map_err`, partitioning results.<br>
**Read**: Book [ch. 9 (*Error Handling*)](https://doc.rust-lang.org/book/ch09-00-error-handling.html); the [`thiserror` docs](https://docs.rs/thiserror).<br>
**Where**: `crates/cerebro-data/src/datasources/claude_registry.rs` (`scan`, `RegistryScan`, `RegistryProblem`).<br>
**Test**: `cargo test -p cerebro-data claude_registry -- --ignored` (1 test: `scan_reports_malformed_files`).

**Why.** `observe()` reads every `~/.claude/sessions/*.json` and drops the files it can't read or
parse, without a word. If Claude Code changes its file format, every session disappears from the
panel and nobody knows why.

**What to build.**

1. `scan()` reads the directory like `observe()` and sorts every entry:

   | Entry | Goes to |
   |---|---|
   | not a `.json` file (e.g. `2.key`) | ignored: neither a session nor a problem |
   | `.json` that can't be read | `problems`, as `(path, RegistryProblem::Unreadable(io error text))` |
   | `.json` that isn't a valid `RegistryEntry` | `problems`, as `(path, RegistryProblem::Malformed(serde error text))` |
   | valid `.json` | `sessions`, converted with `ObservedSession::from` |
   | the directory itself can't be read | the whole call fails: `Err(DomainError::SourceUnavailable(…))`, like `observe()` |

2. Add `thiserror` to `cerebro-data` (`thiserror.workspace = true` in its `Cargo.toml`: the version
   is set in the root `Cargo.toml`) and derive `thiserror::Error` on `RegistryProblem`. Messages:
   `unreadable file: {0}` and `malformed file: {0}`.
3. `observe()` must reuse `scan()` (no second copy of the loop): it returns the scan's sessions and
   prints each problem on stderr as `{path}: {problem}`.
4. Stretch: show the problems in the app. `ClaudeCodeSource` can turn them into the `warning` of
   its `Observation`, which already reaches `Banners.svelte` as a `SourceWarning`. This also fixes a
   side effect of step 3: the app observes every second, so a bad file is printed every second.

**Done when**

- [ ] `scan_reports_malformed_files` passes without `--ignored`, and the other tests of the file still pass;
- [ ] `observe()` no longer contains its own reading loop;
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- Start with a `for` loop and two `Vec`s; once it works, try `Iterator::partition` or `fold`.
- Inside a closure returning `Result`, `map_err` turns an `io::Error` into a `RegistryProblem`,
  then `?` works.
- With `thiserror`, the message goes in an attribute on each variant: `#[error("…: {0}")]`.
</details>

### Exercise 5: statistics with iterators

**Branch**: `features/ex05_session_stats`.<br>
**Goal**: implement `compute_stats`, then print it with `cerebro stats`.<br>
**Concepts**: iterators and adapters (`flat_map`, `fold`, `max_by_key`, `min_by_key`), closures, `BTreeMap`, `Option` combinators.<br>
**Read**: Book [ch. 13 (*Iterators and Closures*)](https://doc.rust-lang.org/book/ch13-00-functional-features.html), [ch. 8.3 (*Hash Maps*)](https://doc.rust-lang.org/book/ch08-03-hash-maps.html).<br>
**Where**: `crates/cerebro-domain/src/entities/stats.rs`, then `crates/cerebro-cli/src/` for the command.<br>
**Test**: `cargo test -p cerebro-domain stats -- --ignored` (3 tests).<br>
**Requires**: exercise 3 (the CLI).

**Why.** A quick overview of what Claude is doing right now: how many agents of each kind, which
session is the busiest, which agent has been running the longest.

**What to build.**

1. `compute_stats(snapshot) -> Stats`, with these rules (also documented on the `Stats` fields):

   | Field | Rule | Empty snapshot |
   |---|---|---|
   | `agents_by_kind` | number of agents per `agent.kind`, over all sessions | empty map |
   | `busiest_session` | id of the session with the most agents; on a tie, the one with the oldest `started_at`. `None` when no session has an agent | `None` |
   | `longest_running_agent` | the agent with the oldest `started_at`, and its age in ms at `snapshot.taken_at` | `None` |

   Rule of the game: **no `for` loop and no `let mut`**. Only iterators.

2. `cerebro stats` prints:

   ```text
   agents       Explore 2, Plan 1
   busiest      cerebro-6d (3 agents)
   longest      Explore in cerebro-6d, running for 12min
   ```

   - kinds in the map's order (alphabetical, that's why it's a `BTreeMap`);
   - sessions by **name**, not by id: `Stats` only holds ids, so look the session up in the
     snapshot;
   - with no agent at all, print `no running agent` instead of the three lines.

**Done when**

- [ ] the 3 tests pass without `--ignored`;
- [ ] `stats.rs` contains no `for` and no `let mut`;
- [ ] `cargo run -p cerebro-cli -- stats` prints the format above, consistent with `cerebro list`;
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- `flat_map` turns "sessions, each with agents" into a single iterator of agents.
- Counting with `fold`: the accumulator is the map, and a closure *parameter* may be `mut`
  (`|mut map, kind| …`): that's allowed by the rule.
- `max_by_key` returns the *last* maximum. To make the oldest session win a tie, put the age in
  the key too: `std::cmp::Reverse` flips an order.
- `Timestamp::since` already saturates at zero.
</details>

### Exercise 6: Terminal.app support

**Branch**: `features/ex06_terminal_app_focus`.<br>
**Goal**: clicking a card focuses the right tab in macOS Terminal, not only in iTerm2.<br>
**Concepts**: implementing a trait for a new type, trait objects (`Box<dyn TabSelector>`), testing with fakes, raw strings.<br>
**Read**: Book [ch. 18.2 (*Trait Objects*)](https://doc.rust-lang.org/book/ch18-02-trait-objects.html); the Reference, [*Raw string literals*](https://doc.rust-lang.org/reference/tokens.html#raw-string-literals), for `r#"…"#`.<br>
**Where**: `crates/cerebro-data/src/strategies/focus/terminal_app.rs`, then `MacFocuser::new` in `focus/mod.rs`.<br>
**Test**: `cargo test -p cerebro-data terminal_app -- --ignored` (3 tests).

**Why.** Clicking a card brings the session's terminal to the front. `MacFocuser` finds the app
hosting the session, then asks each `TabSelector` "do you handle this app?". Only iTerm2 has a
selector today, so Terminal.app users get the app activated but not the right tab.

**What to build.** `TerminalAppTabSelector`, the sibling of `ITermTabSelector` (`iterm.rs`, read it
first):

- `handles(bundle_path)`: `true` when the path is Terminal.app's
  (`/System/Applications/Utilities/Terminal.app`), `false` for any other app (e.g. iTerm).
- `select(runner, tty)`: run an AppleScript with `osascript` through the `runner` (never
  directly). The script looks for the Terminal tab whose `tty` equals the given one (e.g.
  `/dev/ttys007`), selects it, brings its window to the front, and prints `found`; if no tab
  matches, it prints `missing`. `select` returns `Ok(true)` for `found`, `Ok(false)` for
  `missing`, and an error if `osascript` can't run.
- Register the selector in `MacFocuser::new`, next to iTerm's.

The tests use `ScriptedRunner`, a fake `CommandRunner` that records the commands and returns a
canned answer: they never launch AppleScript.

**Done when**

- [ ] the 3 tests pass without `--ignored`;
- [ ] in the app, clicking the card of a session running in Terminal.app selects its tab (try with
  two Terminal windows and several tabs);
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- Compare with `iterm.rs` line by line: only the app name and the script change.
- A raw string `r#"…"#` lets you write the AppleScript with its quotes, unescaped.
- The script:

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
**Test**: `cargo test -p cerebro-data borrowed_parse -- --ignored` (1 test).

**Why.** Every second, Cerebro runs `ps` to check that each session's process is still alive and is
still the same process (a pid can be reused). `parse_ps_line` builds a `Vec` and a new `String` for
every line just to normalize spaces. A `&str` pointing into `ps`'s output is enough.

**What to build.**

1. `parse_ps_line_borrowed(line) -> Option<PsLine<'_>>`: same input as `parse_ps_line`
   (`"  52117 Thu Oct 15  8:43:57 2026"`), but `start` is a slice of `line`, spaces included
   (`"Thu Oct 15  8:43:57 2026"`). `None` when the pid is not a number or the start time is empty.
2. `PsLine::started_at(recorded)`: `true` when `self.start` and `recorded` have the same words,
   whatever the spaces between them (`"Oct 15  8:43"` equals `"Oct 15 8:43"`). No allocation.
3. Switch `PsProcessInspector::inspect` to `parse_ps_line_borrowed`. `ProcessInfo.start` stays a
   `String` because it outlives `ps`'s output: convert there, once per line, with `to_owned()`. Since
   `start` now keeps the raw spaces, check that `same_start` in `datasources/claude_code.rs` still
   matches (it compares word by word).
4. Delete the old `parse_ps_line` and its test, then follow the compiler errors.

**Done when**

- [ ] `borrowed_parse_points_into_the_input` passes without `--ignored` (it checks that `start`
  points into the input, i.e. no copy was made);
- [ ] `parse_ps_line` no longer exists, and every test of the crate passes;
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- `trim_start()` then `split_once(char::is_whitespace)` give two `&str` pointing into the input.
- `a.split_whitespace().eq(b.split_whitespace())` compares word by word without building a `String`.
- Try returning a `PsLine` borrowed from a `String` created inside the function: read the error,
  that's the lesson of the exercise.
</details>

---

## Level 3: Systems programming

### Exercise 8: react instantly with a file watcher

**Branch**: `refactor/ex08_file_watcher`.<br>
**Goal**: update the panel as soon as a session or agent file changes, instead of polling every 1-2 s.<br>
**Concepts**: threads, channels (`std::sync::mpsc`), debouncing, external crates (`notify`).<br>
**Read**: Book [ch. 16 (*Fearless Concurrency*)](https://doc.rust-lang.org/book/ch16-00-concurrency.html); the [`notify` crate docs](https://docs.rs/notify).<br>
**Where**: `src-tauri/src/poller.rs` (and a new `src-tauri/src/watcher.rs` if you like).<br>
**Test**: none prepared; see **Done when**.

**Why.** The poller (`poller.rs`) calls `refresh` every second while the panel is visible, every 2
seconds otherwise. A new subagent can take up to 2 seconds to appear, and Cerebro works even when
nothing changes. The files under `~/.claude/sessions` and `~/.cerebro` change exactly when
something happens: watching them is both faster and cheaper.

**What to build.**

1. Add `notify` to `src-tauri/Cargo.toml` and watch, recursively:
   `~/.claude/sessions` (`Paths::claude_sessions_dir`) and `~/.cerebro` (`Paths::cerebro_dir`). Either may not exist yet at startup (before the first Claude session
   or the first hook): the app must still start and pick them up once they appear.
2. Debounce: a burst of events within 150 ms triggers a **single** `refresh`.
3. Keep a slow safety poll every 10 s: a process can die without touching any file.
4. `refresh` stays the only function that observes and emits: the watcher and the safety poll
   both call it.
5. Ignore changes to `~/.cerebro/assignments.json`: `refresh` writes it itself, which would trigger
   a new refresh, in a loop.

**Done when**

- [ ] starting a subagent shows its character in under half a second;
- [ ] killing a Claude process (`kill <pid>`) removes its card within 10 s;
- [ ] idle CPU does not increase compared to `develop` (Activity Monitor);
- [ ] deleting `~/.cerebro` while the app runs does not crash it;
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- `notify::recommended_watcher(tx)` sends events into an `mpsc::Sender`.
- Debounce loop: block on `rx.recv_timeout(10 s)`; on an event, drain with
  `rx.recv_timeout(150 ms)` until it times out, then refresh once. A timeout on the first wait is
  your safety poll.
- A missing directory can be handled by watching its parent and filtering events by path.
</details>

### Exercise 9: a configuration file

**Branch**: `features/ex09_config_file`.<br>
**Goal**: `~/.cerebro/config.toml` lets the user prefer characters, ban rooms and tune the refresh pace.<br>
**Concepts**: `serde` derive, `#[serde(default)]`, `Default`, the `toml` crate, validating input, passing config through layers.<br>
**Read**: Book [ch. 5 (*Structs*)](https://doc.rust-lang.org/book/ch05-00-structs.html); serde.rs [*Attributes*](https://serde.rs/attributes.html) page.<br>
**Where**: a new `Config` type (reading it touches the file system, so `cerebro-data` or
`cerebro-protocol`), consumed when building the catalog (`src-tauri/src/injector.rs`), by the casting
(`crates/cerebro-domain/src/strategies/casting.rs`) and by the poller.<br>
**Test**: you write them (see below).

**Why.** Characters and rooms are picked pseudo-randomly. Users want their favorites on screen
and some rooms gone.

**What to build.** The file, every key optional:

```toml
favorite_characters = ["wolverine", "storm"]   # ids from assets/characters/characters.json
banned_rooms = ["kitchen"]                     # ids from assets/rooms/rooms.json
refresh_seconds = 5
```

| Key | Effect | Default |
|---|---|---|
| `favorite_characters` | a new session's *lead* character is picked among the favorites first, as long as one is free in its room; otherwise the usual pick | `[]` |
| `banned_rooms` | these rooms are never picked | `[]` |
| `refresh_seconds` | pace of the poll (the safety poll if exercise 8 is done); allowed 1 to 60 | 1 (2 when hidden), or 10 after exercise 8 |

Validation, never fatal: the app always starts.

- missing file → all defaults;
- invalid TOML → all defaults, and a warning on stderr;
- unknown character or room id → ignored, with a warning;
- `refresh_seconds` out of range → default, with a warning;
- banning every room → the ban is ignored (the catalog needs at least one room), with a warning.

**Design rule.** The domain never reads files. `casting.rs` receives plain data (e.g. a filtered
`Catalog` and a list of favorite `CharacterId`s), never a path or a `Config` read from disk. Already
assigned characters and rooms keep their place: the config only affects new picks.

**Done when**

- [ ] tests written **before** the code cover at least: missing file, invalid TOML, unknown id,
  out-of-range `refresh_seconds`, every room banned, a favorite actually picked;
- [ ] editing the file and restarting the app changes the picks of new sessions;
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- `#[serde(default)]` on the struct + `impl Default` gives you "every key optional" for free.
- Add a `config_file()` method to `Paths`, next to `assignments_file()`.
- Filtering banned rooms before `Catalog::new` keeps the casting untouched for that part.
</details>

### Exercise 10: Windows and Linux

**Branch**: `features/ex10_windows_linux`.<br>
**Goal**: Cerebro works outside macOS.<br>
**Concepts**: `#[cfg(target_os = "…")]` vs `cfg!`, platform crates, conditional dependencies in `Cargo.toml`.<br>
**Read**: *The Cargo Book*, ["Platform specific dependencies"](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies); the [`sysinfo` crate](https://docs.rs/sysinfo).<br>
**Where**: `crates/cerebro-data/src/services/process_inspector.rs` (the `cfg!(windows)` branch), `crates/cerebro-data/src/strategies/focus/mod.rs` (`platform_focuser`).<br>
**Test**: you write them, with fake runners like the existing ones.

**Why.** Sessions are listed on every OS, but two pieces are macOS-only: process verification on
Windows trusts the registry blindly (a reused pid shows a ghost session), and clicking a card does
nothing outside macOS (`UnsupportedFocuser`).

**What to build.** Four independent parts; one PR each is fine.

1. **Windows start times**: in `PsProcessInspector::inspect`, get real process start times with
   `sysinfo` (a Windows-only dependency) and format them exactly like Claude Code's `procStart` on
   Windows. Open a registry file on a Windows machine to see that format: don't guess it.
2. **Linux focus**: a focuser that finds the window of the terminal hosting the session
   (`wmctrl -lp` lists windows with their pid; walk up from the Claude pid as `process_tree.rs`
   does on macOS) and raises it (`wmctrl -ia <window id>`). If `wmctrl` is missing, return
   `FocusOutcome::NotFound`, never an error.
3. **Windows focus**: a focuser that brings the terminal window to the front.
4. **Installers in CI**: build the Windows and Linux installers in GitHub Actions with
   `tauri-action`.

**Done when**, for each part: tested with fakes, checked on the real OS, and CI green on the
three platforms.

### Exercise 11 (boss): tmux

**Branch**: `features/ex11_tmux_focus`.<br>
**Goal**: clicking a session running inside tmux selects the right tmux window and pane, then focuses the terminal.<br>
**Concepts**: everything above, plus parsing command output robustly.<br>
**Read**: the [tmux manual](https://man7.org/linux/man-pages/man1/tmux.1.html) (`FORMATS` section).<br>
**Where**: `crates/cerebro-data/src/strategies/focus/` (a new `tmux.rs`) and `MacFocuser::focus`.<br>
**Test**: you write them, with `ScriptedRunner`.

**Why.** Inside tmux, Claude's parent process is the tmux server, not a terminal app.
`find_host_app` finds no `.app` above it, so `MacFocuser::focus` returns `NotFound` and nothing
happens.

**What to build.** When `find_host_app` finds nothing:

1. Get the session's tty (`tty_of`) and look for the tmux pane owning it:
   `tmux list-panes -a -F '#{pane_tty} #{session_name}:#{window_index}.#{pane_index}'`.
2. No match (or tmux not installed): return `NotFound` as today.
3. Match: `tmux select-window -t <target>` then `tmux select-pane -t <target>`.
4. Find the terminal displaying that tmux session:
   `tmux list-clients -F '#{client_tty} #{client_pid} #{session_name}'`, then run the usual host app
   lookup from that client's pid, and activate it (and its tab, through the `TabSelector`s).
5. Return `FocusOutcome::Tab` when everything worked, `App` if only the terminal was activated.

**Done when**: tests cover "no tmux", "no matching pane", "pane found but no client attached" and
the full path, and clicking a tmux session in the app shows the right pane.

### Exercise 12 (no Rust): draw a room

**Branch**: `features/ex12_new_room`.<br>
**Goal**: add a new room to the Institute without writing code.<br>
**Where**: `assets/rooms/` (the PNG) and `assets/rooms/rooms.json`.<br>
**Test**: `cargo test -p cerebro-data shipped_manifests_are_valid`, then run the app.

**What to build.**

1. Draw the room in Aseprite or Piskel. One frame is **96×60 px**. For an animation, put the
   frames side by side in a single horizontal strip: 32 frames make a 3072×60 PNG, like
   `lake.png`.
2. Save it as `assets/rooms/<id>.png` and add an entry to `rooms.json`:

   | Field | Meaning |
   |---|---|
   | `id` | unique, kebab-case, same as the file name |
   | `name` | shown in the UI |
   | `sprite` | `rooms/<id>.png` |
   | `frames` | number of frames in the strip (1 if static) |
   | `fps` | animation speed (8 like the others) |
   | `walkArea` | the rectangle, in pixels inside one frame, where characters may walk: `x`, `y`, `width`, `height` |

3. Run the app until your room shows up (open sessions until it's picked, or ban the other rooms
   if exercise 9 is done).

**Done when**: the room appears, animated if it should be, and characters stay inside the floor.

**Then think about it.** `shipped_manifests_are_valid` only checks that the JSON parses and that
there are at least 20 rooms. A `walkArea` outside the frame, a missing PNG or a wrong frame count
would pass. Bonus: strengthen the test so it catches them.

## Level 4: Async

### Exercise 13: `cerebro serve`, the phone page without the tray app

**Branch**: `features/ex13_cerebro_serve`.<br>
**Goal**: `cerebro serve` serves the phone page from a terminal: on a Linux box without a desktop, over SSH, or to test the page without the app.<br>
**Concepts**: `async`/`await`, the Tokio runtime, `spawn_blocking`, `tokio::select!`, intervals, graceful shutdown.<br>
**Read**: the [Tokio tutorial](https://tokio.rs/tokio/tutorial) (*Hello Tokio*, *Spawning*, *Channels*, *Select*, *Graceful shutdown* in *Topics*); the [Async Book](https://rust-lang.github.io/async-book/), chapters 1 and 2.<br>
**Where**: `crates/cerebro-cli/src/main.rs` (and a new `serve.rs`), `crates/cerebro-cli/src/injector.rs`, `crates/cerebro-cli/Cargo.toml`.<br>
**Test**: none prepared; see **Done when**.

**Why.** The tray app serves phones on its own (`src-tauri/src/mobile.rs`). The server itself
lives in `crates/cerebro-web` and knows nothing about Tauri: any program can host it. Hosting it
from the CLI is the smallest real async program you can write in this codebase: a timer, a
blocking call to keep off the async threads, a server running beside it, and Ctrl-C to stop it
all cleanly.

**What to build.**

1. `cerebro serve [--local] [--dist <dir>]`. `--dist` defaults to `dist` (built by
   `npm run build:ui`). If `<dir>/index.html` is missing, print
   `no UI in <dir>: run npm run build:ui first` and exit with code 1.
2. Addresses:
   - with `--local`, serve on `127.0.0.1` only, to test with a browser on the same machine;
   - otherwise, ask the private network (`ManageNetwork`: add it to the CLI injector like
     `src-tauri/src/injector.rs` does). `Online` with addresses: serve on each of them. Anything
     else: print why (`ZeroTier is stopped`, `ZeroTier is offline`, `no private ZeroTier network
     joined`…) and exit with code 1.
   - the port is always `cerebro_web::DEFAULT_PORT`.
3. Use the app's key, so links keep working: `AccessKey::load_or_create` on
   `Paths::mobile_key_file()`. Print one line per link, e.g.
   `http://10.147.17.42:4243/?key=…`.
4. Every second, observe the sessions and publish the result (`Publisher::publish` with a
   `SnapshotDto`), but only when its content changed (`Snapshot::same_content`).
   `ObserveSessions::execute` blocks (files, `ps`): it must not run on an async worker thread.
5. Ctrl-C stops the server, prints `stopped` and exits with code 0, within a second, even with a
   phone page open.
6. The tray app already listens on port 4243. If the port is taken, print
   `port 4243 is already in use: is the Cerebro app running?` and exit with code 1.

**Done when**

- [ ] `cerebro serve --local`, then the printed link in a browser shows the live sessions;
  starting a Claude session adds its card within 2 s;
- [ ] `curl -i http://127.0.0.1:4243/api/snapshot` (no key) answers `401`;
- [ ] Ctrl-C stops it within a second, with the page still open;
- [ ] with the Cerebro app running, it prints the port message instead of crashing;
- [ ] without `--local`, it serves on your ZeroTier address and your phone shows the page;
- [ ] Clippy is silent.

<details><summary>Hints</summary>

- Only `serve` needs async. Rather than `#[tokio::main]` on `main`, build a runtime for it:
  `tokio::runtime::Runtime::new()?.block_on(serve(...))`. Tokio features: `rt-multi-thread`,
  `macros`, `time`, `signal`.
- `WebServer::start` returns an `io::Error`: its `kind()` is `io::ErrorKind::AddrInUse` when the
  port is taken.
- The loop: `let mut tick = tokio::time::interval(Duration::from_secs(1));` then
  `loop { tokio::select! { _ = tick.tick() => { … } _ = tokio::signal::ctrl_c() => break } }`.
- `tokio::task::spawn_blocking` needs a `'static` closure: share the container in an `Arc` and
  clone the `Arc` into it.
- Dropping the `WebServer` stops it too: look at its `Drop` impl and at
  `streams_each_snapshot_then_ends_when_stopped` in `crates/cerebro-web/src/server.rs`.
</details>

---

## After the exercises


- Read `crates/cerebro-domain/src/strategies/casting.rs` and
  `crates/cerebro-domain/src/usecases/observe_sessions.rs` again. You should now understand every line.
- Pick something you found awkward during the exercises and improve it on your own.
- *Rust for Rustaceans* (Jon Gjengset) is the natural next book.
