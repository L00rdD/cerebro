# Learning Rust with Cerebro

Thirteen exercises that make Cerebro better while teaching Rust, from your first function to
cross-platform and async code. Each one is a real improvement: when you finish, the app (or the CLI)
actually uses your code.

Exercises 5 and up are written like real tickets: the problem is described the way a user or a
teammate would report it, and finding where the code goes is part of the work. Each one has:

- a header: branch, concepts, reading, the test command and, if any, the exercises it requires;
- **The need**: the problem as it would be reported, without a single file or type name;
- **Acceptance criteria**: the observable behavior to deliver (output, rules, edge cases). If
  something you need is not written here, it's a bug in this document: open an issue;
- **Constraints**: what must stay true while you change things;
- **Done when**: the checklist to tick before opening the PR;
- **Hints**, four of them, hidden behind a click and ordered from vaguest to most precise: the
  zone of the code, the exact files and types, the trap, the Rust tools. Try for 20 minutes before
  opening the first one, then open the next one only when you are stuck again.

Exercises 1 to 4 keep an older, guided format: their header also gives the goal and the files to
touch (**Where**), **Why** explains the feature, and **What to build** describes the implementation.

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

In exercises 1 to 4, the **Where** line gives the file. From exercise 5 on there is no **Where**:
finding the zone is part of the exercise. Start from what the user sees (a CLI command, a click in
the panel), follow the code from there, and open hint 2 only if you are lost: it names the files.

Inside the file you'll find:

- the code to write, marked `EXERCISE N` and ending in `todo!(…)`. `todo!()` compiles but
  panics when it runs: it's a placeholder;
- at the bottom, a `mod tests` block whose tests carry `#[ignore = "exercise N"]`. Ignored tests
  are skipped by a normal `cargo test`, so the project stays green until you start.

Some exercises (3, 8 to 13) have no prepared tests: their **Done when** says how to check them.
Writing your own tests is never required: where an exercise suggests some, they are optional.

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

The PR targets `develop`. GitHub Actions runs the tests on macOS (Linux and Windows too for PRs
to `main` or a manual run); merge once everything is green.

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

### Exercise 5: `cerebro stats`, what are my agents doing?

**Branch**: `features/ex05_session_stats`.<br>
**Concepts**: iterators and adapters (`flat_map`, `fold`, `max_by_key`, `min_by_key`), closures, `BTreeMap`, `Option` combinators.<br>
**Read**: Book [ch. 13 (*Iterators and Closures*)](https://doc.rust-lang.org/book/ch13-00-functional-features.html), [ch. 8.3 (*Hash Maps*)](https://doc.rust-lang.org/book/ch08-03-hash-maps.html).<br>
**Test**: `cargo test -p cerebro-domain stats -- --ignored` (3 tests).<br>
**Requires**: exercise 3 (the CLI).

**The need.** "When several sessions run subagents at once, I'd like a quick overview from the
terminal: how many agents of each kind are running, which session is the busiest, which agent has
been running the longest. Right now `cerebro stats` only says it's not implemented yet."

**Acceptance criteria**

- [ ] `cerebro stats` prints three lines:

  ```text
  agents       Explore 2, Plan 1
  busiest      cerebro-6d (3 agents)
  longest      Explore in cerebro-6d, running for 12min
  ```

- [ ] The figures follow these rules:

  | Figure | Rule | Nothing running |
  |---|---|---|
  | agents | number of agents per kind, over all sessions | empty |
  | busiest | the session with the most agents; on a tie, the one launched first (oldest start). None when no session has an agent | none |
  | longest | the agent that started first, and its age at the instant of the observation | none |

- [ ] Kinds are listed in alphabetical order.
- [ ] Sessions are shown by **name**, never by their internal id.
- [ ] With no agent at all, `cerebro stats` prints `no running agent` instead of the three lines.
- [ ] The numbers agree with `cerebro list` (same sessions, same agent counts).

**Constraints**

- The figures are computed in the domain, which stays pure (`docs/ARCHITECTURE.md`): no I/O, and
  no clock of its own. "Now" is the instant the observation was taken.
- Rule of the game: the computation uses iterators only, **no `for` loop and no `let mut`**.
- `cerebro list`, `cerebro hooks` and the existing tests keep working.

**Done when**

- [ ] the 3 prepared tests pass without `--ignored` (the `#[ignore = "exercise 5"]` lines are removed);
- [ ] the statistics code contains no `for` and no `let mut`;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean;
- [ ] `cargo run -p cerebro-cli -- stats` prints the format above, consistent with `cerebro list`.

<details><summary>Hint 1: the zone</summary>

Which layer computes numbers from a snapshot, and which one prints them? Computing is pure logic on
entities: the domain, `cerebro-domain`. Printing is presentation: the CLI, `cerebro-cli`, right
next to the `list` command.
</details>

<details><summary>Hint 2: files and types</summary>

- `crates/cerebro-domain/src/entities/stats.rs`: the `Stats` struct, whose fields document the
  rules (`agents_by_kind: BTreeMap<String, usize>`, `busiest_session: Option<SessionId>`,
  `longest_running_agent: Option<(AgentId, u64)>`, the age in ms at `snapshot.taken_at`), and the
  stub `compute_stats(snapshot: &Snapshot) -> Stats` (`EXERCISE 5`, `todo!`). The tests at the
  bottom: `empty_snapshot_gives_empty_stats`, `counts_and_records`, `tie_goes_to_the_oldest_session`.
- The data: `agent.kind` for the kind, `session.started_at` and `agent.started_at` for the starts,
  `snapshot.taken_at` for "now". `compute_stats` is re-exported by `cerebro_domain::entities`.
- `crates/cerebro-cli/src/main.rs`: the last arm of `match command`, which prints
  `not implemented yet: see EXERCISES.md, exercise 5`.
</details>

<details><summary>Hint 3: the trap</summary>

- `max_by_key` returns the *last* maximum. A real snapshot lists sessions by launch time, so on a
  tie the newest one wins: the wrong one. The prepared tie test lists the newest session first, so
  it can pass by luck: don't rely on the order.
- A plain "max over sessions" still returns a session when every session has 0 agents: `busiest`
  must be `None` then.
- `Stats` only holds ids. To print a session's name, and the kind and session of the longest
  agent, look them up in the snapshot.
- Don't read a clock for the age: `taken_at` already is "now". `Timestamp::since` already
  saturates at zero.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- `flat_map` turns "sessions, each with agents" into a single iterator of agents.
- Counting with `fold`: the accumulator is the map, and a closure *parameter* may be `mut`
  (`|mut map, kind| …`): that's allowed by the rule. `map.entry(key).or_insert(0)` does the rest.
- To make the oldest session win a tie with `max_by_key`, put the age in the key too:
  `std::cmp::Reverse` flips an order. `min_by_key` returns the *first* minimum.
- `humanize_duration` (`cerebro_domain::time`) gives the `12min` of the last line.
</details>

### Exercise 6: clicking a card doesn't find my Terminal.app tab

**Branch**: `features/ex06_terminal_app_focus`.<br>
**Concepts**: implementing a trait for a new type, trait objects (`Box<dyn TabSelector>`), testing with fakes, raw strings.<br>
**Read**: Book [ch. 18.2 (*Trait Objects*)](https://doc.rust-lang.org/book/ch18-02-trait-objects.html); the Reference, [*Raw string literals*](https://doc.rust-lang.org/reference/tokens.html#raw-string-literals), for `r#"…"#`.<br>
**Test**: `cargo test -p cerebro-data terminal_app -- --ignored` (3 tests).

**The need.** "I use the macOS Terminal, not iTerm2. When I click a session's card, Terminal comes
to the front, but on whatever tab was active, not the one running that session. My colleagues on
iTerm2 get the right tab."

**Acceptance criteria**

- [ ] Clicking the card of a session running in Terminal.app selects the Terminal tab whose tty is
  the session's (e.g. `/dev/ttys007`), brings its window to the front and activates Terminal.
- [ ] Terminal.app is recognized by its path, `/System/Applications/Utilities/Terminal.app`. Any
  other app (e.g. `/Applications/iTerm.app`) is not handled by this new support.
- [ ] No tab matches: the click falls back to what it does today, activating Terminal.
- [ ] The AppleScript can't run: it's an error, never a silent success.
- [ ] iTerm2 keeps working exactly as before.

**Constraints**

- The AppleScript runs with `osascript` through the app's command runner, never directly, so tests
  never launch AppleScript.
- Terminal specifics stay in the data layer: the domain only knows that a terminal can be focused
  (`docs/ARCHITECTURE.md`).
- The existing focus tests keep passing.

**Done when**

- [ ] the 3 prepared tests pass without `--ignored` (the `#[ignore = "exercise 6"]` lines are removed);
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean;
- [ ] in the app, clicking the card of a session running in Terminal.app selects its tab (try with
  two Terminal windows and several tabs).

<details><summary>Hint 1: the zone</summary>

Who brings a terminal to the front? Not the domain: it only knows the `TerminalFocuser` port. The
macOS implementation is a strategy of the data layer, `cerebro-data`, in the terminal focus
strategies. iTerm2 is already supported there: its support is your model.
</details>

<details><summary>Hint 2: files and types</summary>

- `crates/cerebro-data/src/strategies/focus/terminal_app.rs`: `TerminalAppTabSelector`, with the
  stubs `handles(bundle_path)` and `select(runner, tty)` (`EXERCISE 6`, `todo!`), and the tests
  `handles_only_terminal_app`, `runs_an_applescript_mentioning_the_tty`, `reports_a_missing_tab`.
- `crates/cerebro-data/src/strategies/focus/iterm.rs`: `ITermTabSelector`, its sibling. Read it first.
- `crates/cerebro-data/src/strategies/focus/mod.rs`: the `TabSelector` trait, and `MacFocuser::new`
  (comment `EXERCISE 6 adds…`), where the selector is registered next to iTerm's. `MacFocuser`
  finds the app hosting the session, then asks each `TabSelector` "do you handle this app?".
- The contract of `select`: the script prints `found` or `missing`; `select` returns `Ok(true)`
  for `found`, `Ok(false)` for `missing`, and an error if `osascript` can't run.
- The tests use `ScriptedRunner` (`focus/mod.rs`, module `fake`), a fake `CommandRunner` that
  records the commands and returns a canned answer.
</details>

<details><summary>Hint 3: the trap</summary>

- The prepared tests call the selector directly. Forget to register it in `MacFocuser::new` and
  they all pass, while a click still only activates Terminal: the manual check is what catches it.
- "Can't run" has two faces: the runner returns `Err` (no `osascript`), or `osascript` runs and
  fails (`output.success` is `false`). `iterm.rs` turns both into `DomainError::Platform`.
- `osascript` ends its output with a newline (`"found\n"` in the tests): trim before comparing.
- Terminal's AppleScript differs from iTerm2's: the application is called `"Terminal"`, and the
  `tty` belongs to the tab (iTerm2 has a `sessions` level below tabs). `set index of w to 1`
  reorders windows but does not bring Terminal forward: `activate` does.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- Compare with `iterm.rs` line by line: only the app name and the script change.
- A raw string `r#"…"#` lets you write the AppleScript with its quotes, unescaped, and `format!`
  can still insert `{tty}` in it.
- `str::ends_with`, `map_err`, `?`; `Box<dyn TabSelector>` in the selectors list.
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

### Exercise 7: stop copying the `ps` output every second

**Branch**: `refactor/ex07_borrowed_ps_parsing`.<br>
**Concepts**: references, lifetimes (`PsLine<'a>`), `&str` slicing, `split_whitespace`, `Iterator::eq`.<br>
**Read**: Book [ch. 4 (*Understanding Ownership*)](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html), [ch. 10.3 (*Validating References with Lifetimes*)](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).<br>
**Test**: `cargo test -p cerebro-data borrowed_parse -- --ignored` (1 test).

**The need.** A teammate: "Every second, Cerebro runs `ps` to check that each session's process is
still alive and is still the same process (a pid can be reused). Reading that code, I saw that every
line of `ps` output is cut into a list and glued back into a brand new string, just to normalize
the spaces in the start time. A view into `ps`'s own output would do: can we parse it without
allocating?"

**Acceptance criteria**

- [ ] Parsing `"  52117 Thu Oct 15  8:43:57 2026"` gives the pid `52117` and the start time
  `"Thu Oct 15  8:43:57 2026"`: a slice of the input line, spaces included, not a copy.
- [ ] A line whose pid is not a number, or whose start time is empty, is rejected (e.g. `garbage`).
- [ ] A parsed start time equals a recorded one when they have the same words, whatever the spaces
  between them: `"Oct 15  8:43"` equals `"Oct 15 8:43"`, and `8:43:58` differs from `8:43:57`.
  The comparison allocates nothing.
- [ ] Sessions are filtered exactly as before: live sessions stay listed and a recycled pid is
  still rejected, even though the start time now keeps its raw spaces.
- [ ] Only the borrowing parser is left: the old copying parser and its test are deleted.

**Constraints**

- It's a refactor: no visible behavior changes, and every test of the crate passes.
- The start time kept for each process outlives `ps`'s output, so it stays an owned string: it is
  converted there, once per line.

**Done when**

- [ ] `borrowed_parse_points_into_the_input` passes without `--ignored`, and its `#[ignore]` line is
  removed (it checks that `start` points into the input, i.e. no copy was made);
- [ ] the old parser no longer exists, and every test of the crate passes;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean.

<details><summary>Hint 1: the zone</summary>

Who runs `ps`? An OS-facing service of the data layer, `cerebro-data`, used by the Claude Code
source to drop sessions whose process is gone or was replaced.
</details>

<details><summary>Hint 2: files and types</summary>

- `crates/cerebro-data/src/services/process_inspector.rs`:
  - `PsLine<'a>` (`pid`, `start: &'a str`) and its stub `started_at(recorded)` (`EXERCISE 7`, `todo!`);
  - the stub `parse_ps_line_borrowed(line) -> Option<PsLine<'_>>`: same input as the old one;
  - the old `parse_ps_line` and its test `parses_lines`, to delete at the end;
  - `PsProcessInspector::inspect`, to switch to `parse_ps_line_borrowed`. `ProcessInfo.start` stays
    a `String`: convert with `to_owned()`.
- `crates/cerebro-data/src/datasources/claude_code.rs`: `same_start`, which compares the recorded
  and actual start times. Check that it still matches now that `start` keeps the raw spaces (it
  compares word by word).
- `crates/cerebro-data/src/services/mod.rs` re-exports both parsers.
</details>

<details><summary>Hint 3: the trap</summary>

- Deleting `parse_ps_line` breaks the `pub use` in `services/mod.rs` and the `parses_lines` test:
  follow the compiler errors.
- Try returning a `PsLine` borrowed from a `String` created inside the function: read the error,
  that's the lesson of the exercise. Any `start` that comes from a new `String` (a `join`, a
  `replace`) also fails the pointer check of the test.
- Trim what follows the pid, so `start` has no stray leading or trailing space and a line ending
  right after the pid counts as an empty start time.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- `trim_start()` then `split_once(char::is_whitespace)` give two `&str` pointing into the input.
- `a.split_whitespace().eq(b.split_whitespace())` compares word by word without building a `String`.
- `str::parse::<u32>().ok()?` and `Option::filter` keep the parser short.
</details>

---

## Level 3: Systems programming

### Exercise 8: new subagents take up to 2 seconds to appear

**Branch**: `refactor/ex08_file_watcher`.<br>
**Concepts**: threads, channels (`std::sync::mpsc`), debouncing, external crates (`notify`).<br>
**Read**: Book [ch. 16 (*Fearless Concurrency*)](https://doc.rust-lang.org/book/ch16-00-concurrency.html); the [`notify` crate docs](https://docs.rs/notify).<br>
**Test**: none prepared; see **Done when**.

**The need.** "When a session starts a subagent, its character can take up to 2 seconds to show up
in the panel. And Cerebro keeps checking every second while the panel is visible, every 2 seconds
otherwise, even when nothing happens. The files under `~/.claude/sessions` and `~/.cerebro` change
exactly when something happens: reacting to them would be both faster and cheaper."

**Acceptance criteria**

- [ ] A change anywhere under `~/.claude/sessions` or `~/.cerebro` (subfolders included) updates
  the panel right away: starting a subagent shows its character in under half a second.
- [ ] A burst of changes within 150 ms triggers a **single** refresh.
- [ ] A slow safety refresh still runs every 10 s, because a process can die without touching any
  file: killing a Claude process (`kill <pid>`) removes its card within 10 s.
- [ ] Either folder may not exist yet at startup (before the first Claude session or the first
  hook): the app still starts, and picks it up once it appears.
- [ ] Deleting `~/.cerebro` while the app runs does not crash it.
- [ ] Changes to `~/.cerebro/assignments.json` are ignored: Cerebro writes that file itself during
  a refresh, which would trigger a new refresh, in a loop.
- [ ] Idle CPU does not increase compared to `develop` (Activity Monitor).

**Constraints**

- One function stays the only one that observes and emits: the watcher and the safety refresh
  both call it. The places that already ask for a refresh keep working.
- Deciding *when* to observe is the app's job: the domain and the data layer don't change.

**Done when**

- [ ] starting a subagent shows its character in under half a second;
- [ ] killing a Claude process (`kill <pid>`) removes its card within 10 s;
- [ ] idle CPU does not increase compared to `develop` (Activity Monitor);
- [ ] deleting `~/.cerebro` while the app runs does not crash it;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean.

<details><summary>Hint 1: the zone</summary>

Who decides when to observe? Not the domain, which observes when asked: the desktop app's background
loop, in `src-tauri`.
</details>

<details><summary>Hint 2: files and types</summary>

- `src-tauri/src/poller.rs`: `start`, the thread that loops with `VISIBLE_INTERVAL` (1 s) or
  `HIDDEN_INTERVAL` (2 s) depending on `panel_visible`, and `refresh`, the only function that
  observes and emits. `refresh` is also called from `panel.rs` and `commands.rs`.
- A new `src-tauri/src/watcher.rs` if you like (declare it in `src-tauri/src/lib.rs`).
- `notify` goes in `src-tauri/Cargo.toml`.
- The folders: `Paths::claude_sessions_dir` and `Paths::cerebro_dir`; the file to ignore:
  `Paths::assignments_file`. All in `crates/cerebro-protocol/src/lib.rs`, built with `Paths::detect()`.
</details>

<details><summary>Hint 3: the trap</summary>

- `assignments.json` is written atomically: `write_json_atomic` (`cerebro-protocol`) writes
  `assignments.json.tmp`, then renames it. Ignore only the exact `assignments.json` path and the
  temporary file's events still start the loop.
- The watcher stops when it is dropped: keep it alive as long as the loop runs, not just inside the
  function that creates it.
- Watching a folder that doesn't exist fails. Watch its parent instead and filter events by path.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- `notify::recommended_watcher(tx)` sends events into an `mpsc::Sender`; `watch(path,
  RecursiveMode::Recursive)` covers subfolders.
- Debounce loop: block on `rx.recv_timeout(10 s)`; on an event, drain with
  `rx.recv_timeout(150 ms)` until it times out, then refresh once. A timeout on the first wait is
  your safety poll.
- `Path::starts_with` and `Path::file_name` to filter events by path.
</details>

### Exercise 9: let me choose my characters and rooms

**Branch**: `features/ex09_config_file`.<br>
**Concepts**: `serde` derive, `#[serde(default)]`, `Default`, the `toml` crate, validating input, passing config through layers.<br>
**Read**: Book [ch. 5 (*Structs*)](https://doc.rust-lang.org/book/ch05-00-structs.html); serde.rs [*Attributes*](https://serde.rs/attributes.html) page.<br>
**Test**: none prepared; writing your own is optional (see **Done when**).

**The need.** "Characters and rooms are picked pseudo-randomly. I want my favorites on screen and
some rooms gone, and I'd like to tune how often the panel refreshes. A small file I can edit,
`~/.cerebro/config.toml`, would be perfect."

**Acceptance criteria**

- [ ] `~/.cerebro/config.toml` is read at startup. Every key is optional:

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

- [ ] Validation is never fatal: the app always starts.
  - missing file → all defaults;
  - invalid TOML → all defaults, and a warning on stderr;
  - unknown character or room id → ignored, with a warning;
  - `refresh_seconds` out of range → default, with a warning;
  - banning every room → the ban is ignored (at least one room is needed), with a warning.
- [ ] Already assigned characters and rooms keep their place: the config only affects new picks.
- [ ] Editing the file and restarting the app changes the picks of new sessions.

**Constraints**

- Design rule: the domain never reads files (`docs/ARCHITECTURE.md`). The casting receives plain
  data (e.g. a filtered catalog and a list of favorite character ids), never a path or a config
  read from disk.
- Reading the file touches the file system, so it lives outside the domain.
- The existing casting tests keep passing.

**Done when**

- [ ] editing the file and restarting the app changes the picks of new sessions;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean;
- [ ] optional: tests written **before** the code, covering at least a missing file, invalid TOML,
  an unknown id, an out-of-range `refresh_seconds`, every room banned, and a favorite actually picked.

<details><summary>Hint 1: the zone</summary>

Which layers does a setting cross? Three. Reading the file is I/O: `cerebro-data` or
`cerebro-protocol`. Wiring happens once, at startup: the app's injector, in `src-tauri`. Using the
values belongs to the casting (domain) for the picks, and to the app's background loop for the pace.
</details>

<details><summary>Hint 2: files and types</summary>

- A new `Config` type in `cerebro-data` or `cerebro-protocol`, and a `config_file()` method on
  `Paths` (`crates/cerebro-protocol/src/lib.rs`), next to `assignments_file()`. `toml` goes in that
  crate's `Cargo.toml`.
- `src-tauri/src/injector.rs`, `build()`: builds the catalog with `EmbeddedCatalog.catalog()`
  (`crates/cerebro-data/src/repositories/embedded_catalog.rs`) and hands it to `ObserveSessions`
  through `ObserveSessionsDeps { catalog, .. }`.
- `Catalog::new(characters, rooms)` (`crates/cerebro-domain/src/entities/catalog.rs`) fails when
  either list is empty.
- `crates/cerebro-domain/src/strategies/casting.rs`: `cast_all`, called by `ObserveSessions::execute`
  (`crates/cerebro-domain/src/usecases/observe_sessions.rs`), then `pick_room` and `pick_character`.
  The lead's key is `ActorKey::Lead`.
- The pace: `VISIBLE_INTERVAL` and `HIDDEN_INTERVAL` in `src-tauri/src/poller.rs`.
- The CLI builds its own catalog too (`crates/cerebro-cli/src/injector.rs`): decide whether it
  should honor the file.
</details>

<details><summary>Hint 3: the trap</summary>

- Filtering banned rooms out before `Catalog::new` keeps the casting untouched for that part, but
  `pick_room` and `pick_character` only keep an existing pick if it is still in the catalog, and
  picks are reloaded from `~/.cerebro/assignments.json` at startup. So a session already sitting
  in a newly banned room moves at the next start. Decide
  whether that is acceptable for a ban (and say so in a comment), or pass the ban to the casting
  separately.
- Favorites only change *new* lead picks: a lead or agent that already has a character keeps it.
- "Never fatal" means no `?` that bubbles up to a failed startup: a ban of every room would make
  `Catalog::new` fail and the app would not start.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- `#[serde(default)]` on the struct + `impl Default` gives you "every key optional" for free.
- `toml::from_str`; `std::fs::read_to_string`, whose `io::ErrorKind::NotFound` is the missing file.
- `(1..=60).contains(&value)` for the range; `Vec::retain` to drop unknown or banned ids.
- `pick_free` (`crates/cerebro-domain/src/rng.rs`) picks deterministically in any list, including
  a list of favorites.
</details>

### Exercise 10: Cerebro on Windows and Linux

**Branch**: `features/ex10_windows_linux`.<br>
**Concepts**: `#[cfg(target_os = "…")]` vs `cfg!`, platform crates, conditional dependencies in `Cargo.toml`.<br>
**Read**: *The Cargo Book*, ["Platform specific dependencies"](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies); the [`sysinfo` crate](https://docs.rs/sysinfo).<br>
**Test**: none prepared; tests with fake runners like the existing ones are optional.

**The need.** "Sessions are listed on every OS, but two things only work on macOS. On Windows, a
pid reused by another program shows a ghost session: Cerebro trusts Claude Code's registry blindly.
And outside macOS, clicking a card does nothing. We'd also like installers for Windows and Linux."

**Acceptance criteria.** Four independent parts; one PR each is fine.

1. **Windows start times**
   - [ ] On Windows, a session whose pid now belongs to another process is hidden, like on macOS.
   - [ ] The start time Cerebro gets from Windows is formatted exactly like Claude Code's
     `procStart` on Windows (in `~/.claude/sessions/<pid>.json`). Open a registry file on a
     Windows machine to see that format: don't guess it.
2. **Linux focus**
   - [ ] Clicking a card raises the window of the terminal hosting the session: `wmctrl -lp` lists
     windows with their pid, the terminal is found by walking up the process tree from the Claude
     pid, and `wmctrl -ia <window id>` raises it.
   - [ ] If `wmctrl` is missing, the click reports "not found", never an error.
3. **Windows focus**
   - [ ] Clicking a card brings the terminal window to the front.
4. **Installers in CI**
   - [ ] The Windows and Linux installers are built in GitHub Actions with `tauri-action`.

**Constraints**

- macOS behavior doesn't change.
- A platform-only crate is declared for its platform only (`sysinfo` is a Windows-only dependency).
- The domain doesn't change: platforms are a data-layer concern (`docs/ARCHITECTURE.md`).
- The workspace forbids `unsafe` code.

**Done when**, for each part:

- [ ] checked on the real OS;
- [ ] CI green on the three platforms;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean;
- [ ] optional: tested with fakes, like the existing focus and `ps` tests.

<details><summary>Hint 1: the zone</summary>

Where does Cerebro talk to the OS? In the data layer, `cerebro-data`: the service that inspects
processes, and the strategies that focus a terminal. The installers are a matter for
`.github/workflows/`.
</details>

<details><summary>Hint 2: files and types</summary>

- `crates/cerebro-data/src/services/process_inspector.rs`: `PsProcessInspector::inspect`, whose
  `cfg!(windows)` branch (`EXERCISE 10`) returns `ProcessInfo { start: None }` for every pid.
  The recorded value is `RegistryEntry.proc_start` (`crates/cerebro-data/src/models/registry_entry.rs`).
- `crates/cerebro-data/src/strategies/focus/mod.rs`: `platform_focuser`, which returns `MacFocuser`
  on macOS and `UnsupportedFocuser` (always `FocusOutcome::NotFound`) elsewhere. The walk up the
  process tree on macOS is `find_host_app` in `process_tree.rs`.
- The fakes: `ScriptedRunner` (`focus/mod.rs`, module `fake`) and `FakeRunner` in the tests of
  `process_inspector.rs`. Every external command goes through `CommandRunner`.
- `.github/workflows/release.yml` already runs `tauri-apps/tauri-action` on `ubuntu-22.04` and
  `windows-latest` when a version is tagged: part 4 may only need checking. `.github/workflows/ci.yml`
  runs the Rust job.
</details>

<details><summary>Hint 3: the trap</summary>

- `cfg!(windows)` is an ordinary boolean: both branches are compiled on every OS. Code calling a
  Windows-only crate then fails to compile on macOS: put it behind `#[cfg(windows)]` instead.
- A missing program is not a failed command: `CommandRunner::run` returns `Err`
  (`io::ErrorKind::NotFound`) when `wmctrl` isn't installed. That case must become `NotFound`.
- On macOS, `ps` is run with `TZ=UTC` because Claude Code records `procStart` in UTC. Don't assume
  the same on Windows: check the format and the time zone in a real registry file.
- `unsafe_code = "forbid"` in the root `Cargo.toml`: calling Win32 functions directly is out. Look
  for a safe route for Windows focus.
- On `develop`, the Rust CI job only runs on macOS. The three platforms run for `main` or a manual
  `workflow_dispatch`.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- `[target.'cfg(windows)'.dependencies]` in `Cargo.toml`, and `#[cfg(target_os = "linux")]` on
  items.
- `sysinfo` gives each process its start time in seconds since 1970.
- `str::lines`, `split_whitespace`, `str::parse` for `wmctrl -lp`'s columns.
</details>

### Exercise 11 (boss): clicking a session inside tmux does nothing

**Branch**: `features/ex11_tmux_focus`.<br>
**Concepts**: everything above, plus parsing command output robustly.<br>
**Read**: the [tmux manual](https://man7.org/linux/man-pages/man1/tmux.1.html) (`FORMATS` section).<br>
**Test**: none prepared; tests with `ScriptedRunner` are optional.

**The need.** "I run Claude Code inside tmux. Clicking its card does nothing at all: no terminal
comes forward, and tmux stays on whatever window and pane it was showing."

**Acceptance criteria.** When the session has no terminal app directly above it:

- [ ] Cerebro looks for the tmux pane owning the session's tty:
  `tmux list-panes -a -F '#{pane_tty} #{session_name}:#{window_index}.#{pane_index}'`.
- [ ] No match (or tmux not installed): nothing happens, reported as "not found", as today.
- [ ] Match: `tmux select-window -t <target>` then `tmux select-pane -t <target>`.
- [ ] Cerebro finds the terminal displaying that tmux session:
  `tmux list-clients -F '#{client_tty} #{client_pid} #{session_name}'`, runs the usual terminal app
  lookup from that client's pid, and activates it (and its tab, like any other session).
- [ ] The outcome is "tab" when everything worked, "app" if only the terminal was activated.
- [ ] Sessions outside tmux behave as before.

**Constraints**

- Every tmux call goes through the command runner, so it can be faked.
- The domain doesn't change: the outcomes it knows are enough.

**Done when**

- [ ] clicking a tmux session in the app shows the right pane;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean;
- [ ] optional: tests cover "no tmux", "no matching pane", "pane found but no client attached" and
  the full path.

<details><summary>Hint 1: the zone</summary>

Where does a click on a card end up on macOS? In the macOS focus strategy of the data layer,
`cerebro-data`: the same place as exercise 6.
</details>

<details><summary>Hint 2: files and types</summary>

- A new `crates/cerebro-data/src/strategies/focus/tmux.rs` (declare it in `focus/mod.rs`).
- `MacFocuser::focus` (`focus/mod.rs`): when `find_host_app` finds nothing, it returns
  `FocusOutcome::NotFound` today. Inside tmux, Claude's parent process is the tmux server, not a
  terminal app, so no `.app` is found above it.
- Already there: `tty_of` and `find_host_app` (`process_tree.rs`), `HostApp`, the `TabSelector`s
  and `MacFocuser::activate`. `FocusOutcome::{Tab, App, NotFound}` lives in the domain ports.
- The existing test `unknown_host_is_not_found` uses a tmux host: it must still pass.
</details>

<details><summary>Hint 3: the trap</summary>

- `ScriptedRunner` matches commands by prefix and answers success with an empty output to anything
  it doesn't know: an unexpected tmux command looks like "tmux is there, no pane". A missing tmux
  is a runner `Err`, which `ScriptedRunner` can't produce: that test needs its own fake.
- `CommandRunner::run` doesn't go through a shell: the single quotes of the commands above are shell
  quoting. Pass `-F` and the format as two arguments, without quotes.
- Session names may contain spaces: split each line a limited number of times, from the left.
- "Pane found but no client attached": the pane is selected, but no terminal comes forward. The
  rules above don't name that outcome: pick an honest one and explain it in a comment.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- `str::lines`, `split_once`, `splitn(3, ' ')`.
- `Iterator::find_map` to go from lines to the first match.
- A helper returning `Option` lets `?` handle every "no match"; `let … else` in the caller.
</details>

### Exercise 12 (no Rust): draw a room

**Branch**: `features/ex12_new_room`.<br>
**Test**: `cargo test -p cerebro-data shipped_manifests_are_valid`, then run the app.

**The need.** "The Institute could use a new room. Drawing it and declaring it should be enough,
without writing code."

**Acceptance criteria**

- [ ] The room is drawn (Aseprite or Piskel) at the shared size: one frame is **192×120 px**;
  characters are 24×24 with their feet at the bottom.
- [ ] An animated room puts its frames side by side in a single horizontal strip: 16 frames make a
  3072×120 PNG, like `lake.png`.
- [ ] It is saved as `assets/rooms/<id>.png` and declared in `assets/rooms/rooms.json`:

  | Field | Meaning |
  |---|---|
  | `id` | unique, kebab-case, same as the file name |
  | `name` | shown in the UI |
  | `sprite` | `rooms/<id>.png` |
  | `frames` | number of frames in the strip (1 if static) |
  | `fps` | animation speed (8 like the others) |
  | `walkArea` | the rectangle, in pixels inside one frame, where characters may walk: `x`, `y`, `width`, `height` |

- [ ] The room appears in the app, animated if it should be, and characters stay inside the floor.

**Constraints**

- No code change: art lives in files (`docs/ARCHITECTURE.md`, *Assets*).
- The shipped rooms keep working.

**Done when**

- [ ] `cargo test -p cerebro-data shipped_manifests_are_valid` passes, and so does `npm run test:rust`;
- [ ] you ran the app until your room showed up (open sessions until it's picked, or ban the other
  rooms if exercise 9 is done);
- [ ] the room appears, animated if it should be, and characters stay inside the floor.

**Then think about it.** `shipped_manifests_are_valid` only checks that the JSON parses and that
there are at least 20 rooms. A `walkArea` outside the frame, a missing PNG or a wrong frame count
would pass. Bonus: strengthen the test so it catches them.

<details><summary>Hint 1: the zone</summary>

Where does the art live? Not in code: in `assets/`, shared by the Rust backend and the UI.
</details>

<details><summary>Hint 2: files and types</summary>

- `assets/rooms/<id>.png` and `assets/rooms/rooms.json` (`frameWidth`, `frameHeight`, `rooms`).
- The backend compiles `rooms.json` into the binary: `EmbeddedCatalog`
  (`crates/cerebro-data/src/repositories/embedded_catalog.rs`), whose test is
  `shipped_manifests_are_valid`. The UI loads it with the sprites in `ui/src/lib/assets/catalog.ts`.
- `scripts/art/rooms.py` draws every shipped room: you can add a function there, registered with
  `@room(…)`.
</details>

<details><summary>Hint 3: the trap</summary>

- `walkArea` is measured inside **one frame**, not inside the whole strip.
- `frames` must match the strip: its width divided by 192.
- `sprite` is relative to `assets/`: `rooms/<id>.png`, not `assets/rooms/<id>.png`.
- Running `scripts/art/rooms.py` without arguments rewrites the whole `rooms.json` from the rooms
  the script knows (in its `ORDER` list): an entry added only by hand is lost.
- Rooms are picked pseudo-randomly from the session id: yours may not show up on the first try.
</details>

<details><summary>Hint 4: the tools</summary>

- `file assets/rooms/lake.png` prints the size of a PNG.
- For the bonus test: a PNG stores its width and height as two big-endian `u32` at bytes 16 to 24
  (`u32::from_be_bytes`); `Path::exists` checks the sprite is there.
</details>

## Level 4: Async

### Exercise 13: `cerebro serve`, the phone page without the tray app

**Branch**: `features/ex13_cerebro_serve`.<br>
**Concepts**: `async`/`await`, the Tokio runtime, `spawn_blocking`, `tokio::select!`, intervals, graceful shutdown.<br>
**Read**: the [Tokio tutorial](https://tokio.rs/tokio/tutorial) (*Hello Tokio*, *Spawning*, *Channels*, *Select*, *Graceful shutdown* in *Topics*); the [Async Book](https://rust-lang.github.io/async-book/), chapters 1 and 2.<br>
**Test**: none prepared; see **Done when**.

**The need.** "I'd like the phone page without the tray app: on a Linux box without a desktop, over
SSH, or just to test the page without launching the app. `cerebro serve` exists, but it only says
it's not implemented yet."

**Acceptance criteria**

- [ ] The command is `cerebro serve [--local] [--dist <dir>]`. `--dist` defaults to `dist` (built
  by `npm run build:ui`). If `<dir>/index.html` is missing, it prints
  `no UI in <dir>: run npm run build:ui first` and exits with code 1.
- [ ] Addresses:
  - with `--local`, it serves on `127.0.0.1` only, to test with a browser on the same machine;
  - otherwise, it asks the private network. Online with addresses: it serves on each of them.
    Anything else: it prints why (`ZeroTier is stopped`, `ZeroTier is offline`, `no private
    ZeroTier network joined`…) and exits with code 1;
  - the port is always the app's phone port, 4243.
- [ ] Links keep working: it uses the app's key (`~/.cerebro/mobile-key`, created if missing) and
  prints one line per link, e.g. `http://10.147.17.42:4243/?key=…`.
- [ ] Every second, it observes the sessions and updates the page, but only when the content
  changed: starting a Claude session adds its card within 2 s.
- [ ] The API still wants the key: `curl -i http://127.0.0.1:4243/api/snapshot` (no key) answers `401`.
- [ ] Ctrl-C stops the server, prints `stopped` and exits with code 0, within a second, even with a
  phone page open.
- [ ] The tray app already listens on port 4243. If the port is taken, it prints
  `port 4243 is already in use: is the Cerebro app running?` and exits with code 1.

**Constraints**

- Observing sessions blocks (files, `ps`): it must not run on an async worker thread.
- Only `serve` is async: `list`, `stats` and `hooks` don't change.
- The CLI reuses the existing phone server as is, without any Tauri (`docs/ARCHITECTURE.md`).

**Done when**

- [ ] `cerebro serve --local`, then the printed link in a browser shows the live sessions;
  starting a Claude session adds its card within 2 s;
- [ ] `curl -i http://127.0.0.1:4243/api/snapshot` (no key) answers `401`;
- [ ] Ctrl-C stops it within a second, with the page still open;
- [ ] with the Cerebro app running, it prints the port message instead of crashing;
- [ ] without `--local`, it serves on your ZeroTier address and your phone shows the page;
- [ ] `npm run test:rust`, `npm run lint:rust` and `cargo fmt --all` are clean.

<details><summary>Hint 1: the zone</summary>

Who can host the phone page? The tray app does it today (`src-tauri/src/mobile.rs`), but the server
itself lives in `crates/cerebro-web` and knows nothing about Tauri: any program can host it. Here
the host is the CLI, `cerebro-cli`. It's the smallest real async program you can write in this
codebase: a timer, a blocking call to keep off the async threads, a server running beside it, and
Ctrl-C to stop it all cleanly.
</details>

<details><summary>Hint 2: files and types</summary>

- `crates/cerebro-cli/src/main.rs`: the `"serve"` arm, and the argument reading, which only looks at
  the first argument today. A new `serve.rs` for the command itself.
- `crates/cerebro-cli/src/injector.rs`: add `ManageNetwork` to `Container`, built like
  `src-tauri/src/injector.rs` does (`ZeroTierNetwork::new(runner)`), and the key file,
  `Paths::mobile_key_file()`.
- `crates/cerebro-cli/Cargo.toml`: add `cerebro-web` and `tokio`.
- From `cerebro_web`: `WebServer::start(addresses, publisher, assets, key)` (async),
  `Publisher::new` and `Publisher::publish`, `DirAssets::new(dir)`, `AccessKey::load_or_create`,
  `AccessKey::as_str`, `DEFAULT_PORT`, and `cerebro_web::dto::SnapshotDto` (`From<&Snapshot>`).
- From the domain: `ManageNetwork::status()` returns a `NetworkStatus` (`NotInstalled`, `Stopped`,
  `Offline`, `Online(Vec<IpAddr>)`) and `ManageNetwork::name()` gives `ZeroTier`;
  `ObserveSessions::execute`; `Snapshot::same_content`.
- `MobileDto::new` in `src-tauri/src/dto.rs` builds the same links: `http://{address}/?key=…`.
</details>

<details><summary>Hint 3: the trap</summary>

- `tokio::task::spawn_blocking` needs a `'static` closure: share the container in an `Arc` and
  clone the `Arc` into it.
- `tokio::signal::ctrl_c()` only sees a Ctrl-C pressed after it is first polled. Call it inside
  the loop and a Ctrl-C pressed while the sessions are being observed is lost: create it once,
  before the loop, and pin it.
- Publish the first snapshot too: there is nothing to compare it with yet, and until then
  `/api/snapshot` answers `204`.
- `WebServer::start` puts the address in front of the error message: test the error's kind, not
  its text.
- `Online` with no address is the "no private network joined" case.
</details>

<details><summary>Hint 4: the Rust tools</summary>

- Only `serve` needs async. Rather than `#[tokio::main]` on `main`, build a runtime for it:
  `tokio::runtime::Runtime::new()?.block_on(serve(...))`. Tokio features: `rt-multi-thread`,
  `macros`, `time`, `signal`.
- `WebServer::start` returns an `io::Error`: its `kind()` is `io::ErrorKind::AddrInUse` when the
  port is taken.
- The loop: `let mut tick = tokio::time::interval(Duration::from_secs(1));` then
  `loop { tokio::select! { _ = tick.tick() => { … } _ = tokio::signal::ctrl_c() => break } }`
  (with the Ctrl-C future created before the loop, see hint 3: `tokio::pin!`, then `&mut` it in
  the `select!`).
- Dropping the `WebServer` stops it too: look at its `Drop` impl and at
  `streams_each_snapshot_then_ends_when_stopped` in `crates/cerebro-web/src/server.rs`.
</details>

---

## After the exercises


- Read `crates/cerebro-domain/src/strategies/casting.rs` and
  `crates/cerebro-domain/src/usecases/observe_sessions.rs` again. You should now understand every line.
- Pick something you found awkward during the exercises and improve it on your own.
- *Rust for Rustaceans* (Jon Gjengset) is the natural next book.
