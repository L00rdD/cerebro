//! `cerebro`: command line companion (EXERCISE 3).
//!
//! Target usage:
//! ```text
//! cerebro list     one line per live session (uses `Display for Session`, exercise 2)
//! cerebro stats    agents by type, busiest session… (uses `compute_stats`, exercise 5)
//! cerebro hooks    hook installation status
//! ```
//!
//! Reuse the same adapters as the app: look at `src-tauri/src/injector.rs`.

mod injector;

use std::process::ExitCode;

use cerebro_domain::entities::Snapshot;
use cerebro_domain::ports::HookSetup;
use cerebro_domain::time::humanize_duration;
use cerebro_domain::usecases::ManageHooks;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let Some(command @ ("list" | "stats" | "hooks")) = command.as_deref() else {
        eprintln!("usage: cerebro <list|stats|hooks>");
        return ExitCode::from(2);
    };
    let container = match injector::build() {
        Ok(container) => container,
        Err(error) => {
            eprintln!("cannot start: {error}");
            return ExitCode::FAILURE;
        }
    };
    match command {
        "list" => list(&container.observe.execute()),
        "hooks" => hooks(&container.hooks),
        _ => {
            eprintln!("not implemented yet: see EXERCISES.md, exercise 5");
            ExitCode::FAILURE
        }
    }
}

fn hooks(hooks: &ManageHooks) -> ExitCode {
    match hooks.status() {
        Ok(HookSetup::Installed) => {
            println!("hooks: installed");
            ExitCode::SUCCESS
        }
        Ok(HookSetup::Missing) => {
            println!("hooks: missing (subagents and the waiting status are not tracked)");
            ExitCode::FAILURE
        }
        Ok(HookSetup::Outdated) => {
            println!("hooks: outdated (they point to another cerebro-hook or miss some events)");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("cannot read the hooks: {error}");
            ExitCode::FAILURE
        }
    }
}

fn list(snapshot: &Snapshot) -> ExitCode {
    for warning in &snapshot.warnings {
        eprintln!("{}: {}", warning.tool, warning.message);
    }
    if snapshot.scenes.is_empty() {
        println!("no live session");
        return ExitCode::SUCCESS;
    }

    // name, status, agents, status age, folder
    let rows: Vec<[String; 5]> = snapshot
        .scenes
        .iter()
        .map(|scene| {
            let session = &scene.session;
            let agent_count = session.agents.len();
            let plural = if agent_count == 1 { "" } else { "s" };
            [
                session.name.clone(),
                session.status.to_string(),
                format!("{agent_count} agent{plural}"),
                humanize_duration(snapshot.taken_at.since(session.status_since)),
                session.cwd.display().to_string(),
            ]
        })
        .collect();

    // First pass: each column is as wide as its longest cell.
    let mut widths = [0; 5];
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    // Second pass: pad every cell; the folder is last, so it needs no padding.
    for [name, status, agents, age, cwd] in &rows {
        println!(
            "{name:<name_w$}  {status:<status_w$}  {agents:>agents_w$}  {age:>age_w$}  {cwd}",
            name_w = widths[0],
            status_w = widths[1],
            agents_w = widths[2],
            age_w = widths[3],
        );
    }
    ExitCode::SUCCESS
}
