//! `cerebro`: command line companion (EXERCISE 3).
//!
//! Target usage:
//! ```text
//! cerebro list     one line per live session (uses `Display for Session`, exercise 2)
//! cerebro stats    agents by type, busiest session… (uses `compute_stats`, exercise 5)
//! cerebro hooks    hook installation status
//! cerebro serve    the phone page, without the tray app (async, exercise 13)
//! ```
//!
//! Reuse the same adapters as the app: look at `src-tauri/src/injector.rs`.

mod injector;

use std::process::ExitCode;

use cerebro_domain::entities::{Session, Snapshot};
use cerebro_domain::ports::HookSetup;
use cerebro_domain::time::humanize_duration;
use cerebro_domain::usecases::ManageHooks;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let Some(command @ ("list" | "stats" | "hooks" | "serve")) = command.as_deref() else {
        eprintln!("usage: cerebro <list|stats|hooks|serve>");
        return ExitCode::from(2);
    };
    match injector::build() {
        Ok(container) => run(command, &container),
        Err(error) => {
            eprintln!("cannot start: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: &str, container: &injector::Container) -> ExitCode {
    match command {
        "list" => list(&container.observe.execute()),
        "hooks" => hooks(&container.hooks),
        "serve" => {
            eprintln!("not implemented yet: see EXERCISES.md, exercise 13");
            ExitCode::FAILURE
        }
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
    for line in table(snapshot) {
        println!("{line}");
    }
    ExitCode::SUCCESS
}

/// One aligned line per live session, or a single line saying there is none.
fn table(snapshot: &Snapshot) -> Vec<String> {
    if snapshot.scenes.is_empty() {
        return vec!["no live session".to_owned()];
    }

    // name, status, agents, status age, folder
    let rows: Vec<[String; 5]> = snapshot
        .scenes
        .iter()
        .map(|scene| row(snapshot, &scene.session))
        .collect();
    let widths = column_widths(&rows);

    // Pad every cell; the folder is last, so it needs no padding.
    rows.iter()
        .map(|[name, status, agents, age, cwd]| {
            format!(
                "{name:<name_w$}  {status:<status_w$}  {agents:>agents_w$}  {age:>age_w$}  {cwd}",
                name_w = widths[0],
                status_w = widths[1],
                agents_w = widths[2],
                age_w = widths[3],
            )
        })
        .collect()
}

fn row(snapshot: &Snapshot, session: &Session) -> [String; 5] {
    let agent_count = session.agents.len();
    let plural = if agent_count == 1 { "" } else { "s" };
    [
        session.name.clone(),
        session.status.to_string(),
        format!("{agent_count} agent{plural}"),
        humanize_duration(snapshot.taken_at.since(session.status_since)),
        session.cwd.display().to_string(),
    ]
}

/// Each column is as wide as its longest cell.
fn column_widths(rows: &[[String; 5]]) -> [usize; 5] {
    let mut widths = [0; 5];
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }
    widths
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use cerebro_domain::entities::{
        Agent, AgentId, Cast, CharacterId, RoomId, Scene, SessionId, SessionStatus, ToolId,
    };
    use cerebro_domain::time::Timestamp;

    use super::*;

    fn scene(name: &str, status: SessionStatus, agents: usize, since_ms: u64, cwd: &str) -> Scene {
        let agent = |n: usize| Agent {
            id: AgentId::new(format!("a{n}")),
            kind: "Explore".into(),
            started_at: Timestamp(0),
        };
        Scene {
            session: Session {
                id: SessionId::new(name),
                tool: ToolId::new("claude-code"),
                pid: None,
                name: name.into(),
                cwd: PathBuf::from(cwd),
                status,
                started_at: Timestamp(0),
                status_since: Timestamp(since_ms),
                remote_url: None,
                agents: (0..agents).map(agent).collect(),
            },
            cast: Cast {
                room: RoomId::new("office"),
                lead: CharacterId::new("storm"),
                agents: Vec::new(),
            },
        }
    }

    fn snapshot(scenes: Vec<Scene>) -> Snapshot {
        Snapshot {
            scenes,
            warnings: Vec::new(),
            taken_at: Timestamp(3_600_000),
        }
    }

    #[test]
    fn no_session_says_so() {
        assert_eq!(table(&snapshot(Vec::new())), vec!["no live session"]);
    }

    #[test]
    fn columns_are_aligned_on_their_longest_cell() {
        let lines = table(&snapshot(vec![
            scene("personal", SessionStatus::Busy, 1, 3_595_000, "/tmp/a"),
            scene("w", SessionStatus::Waiting, 12, 0, "/tmp/longer"),
        ]));
        assert_eq!(
            lines,
            vec![
                "personal  busy       1 agent    5s  /tmp/a",
                "w         waiting  12 agents  1h00  /tmp/longer",
            ]
        );
    }
}
