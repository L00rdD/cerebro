//! Turns Claude Code hook events into Cerebro state files.

use std::fs;
use std::io;

use cerebro_protocol::{
    FORMAT_VERSION, HookAgent, HookSession, HookStatus, Paths, read_json, sanitize_id,
    write_json_atomic,
};
use serde_json::Value;

/// What `handle` did, mostly useful for tests.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    SessionUpdated(HookStatus),
    SessionUnchanged,
    SessionRemoved,
    AgentStarted,
    AgentStopped,
    Ignored,
}

pub fn handle(raw_event: &str, paths: &Paths, now_ms: u64) -> io::Result<Outcome> {
    let event: Value = serde_json::from_str(raw_event)?;
    let name = event["hook_event_name"].as_str().unwrap_or_default();
    let Some(session_id) = event["session_id"].as_str().and_then(sanitize_id) else {
        return Ok(Outcome::Ignored);
    };
    let cwd = event["cwd"].as_str().unwrap_or_default();

    match name {
        "SessionStart" | "Stop" => set_status(paths, &session_id, cwd, HookStatus::Idle, now_ms),
        "UserPromptSubmit" | "PostToolUse" => {
            set_status(paths, &session_id, cwd, HookStatus::Busy, now_ms)
        }
        "Notification" if is_permission_prompt(&event) => {
            set_status(paths, &session_id, cwd, HookStatus::Waiting, now_ms)
        }
        "SessionEnd" => {
            remove_if_exists(&paths.hook_session_file(&session_id))?;
            match fs::remove_dir_all(paths.session_agents_dir(&session_id)) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
                _ => {}
            }
            Ok(Outcome::SessionRemoved)
        }
        "SubagentStart" => {
            let Some(agent_id) = event["agent_id"].as_str().and_then(sanitize_id) else {
                return Ok(Outcome::Ignored);
            };
            let agent = HookAgent {
                version: FORMAT_VERSION,
                agent_type: event["agent_type"]
                    .as_str()
                    .unwrap_or("general-purpose")
                    .to_owned(),
                agent_id,
                started_at_ms: now_ms,
            };
            write_json_atomic(&paths.agent_file(&session_id, &agent.agent_id), &agent)?;
            Ok(Outcome::AgentStarted)
        }
        "SubagentStop" => {
            let Some(agent_id) = event["agent_id"].as_str().and_then(sanitize_id) else {
                return Ok(Outcome::Ignored);
            };
            remove_if_exists(&paths.agent_file(&session_id, &agent_id))?;
            Ok(Outcome::AgentStopped)
        }
        _ => Ok(Outcome::Ignored),
    }
}

/// Only permission prompts mean "blocked"; idle reminders are ignored.
fn is_permission_prompt(event: &Value) -> bool {
    match event["notification_type"].as_str() {
        Some(kind) => kind == "permission_prompt",
        None => event["message"]
            .as_str()
            .is_some_and(|message| message.to_lowercase().contains("permission")),
    }
}

fn set_status(
    paths: &Paths,
    session_id: &str,
    cwd: &str,
    status: HookStatus,
    now_ms: u64,
) -> io::Result<Outcome> {
    let file = paths.hook_session_file(session_id);
    let previous = read_json::<HookSession>(&file).ok();

    // PostToolUse fires on every tool call: skip the write when nothing changes.
    if previous
        .as_ref()
        .is_some_and(|previous_record| previous_record.status == status)
    {
        return Ok(Outcome::SessionUnchanged);
    }

    let record = HookSession {
        version: FORMAT_VERSION,
        session_id: session_id.to_owned(),
        cwd: match &previous {
            Some(previous_record) if cwd.is_empty() => previous_record.cwd.clone(),
            _ => cwd.to_owned(),
        },
        status,
        started_at_ms: previous.map_or(now_ms, |previous_record| previous_record.started_at_ms),
        status_since_ms: now_ms,
    };
    write_json_atomic(&file, &record)?;
    Ok(Outcome::SessionUpdated(status))
}

fn remove_if_exists(path: &std::path::Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(paths: &Paths, event: &Value, now: u64) -> Outcome {
        handle(&event.to_string(), paths, now).unwrap()
    }

    fn setup() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path());
        (dir, paths)
    }

    #[test]
    fn session_lifecycle() {
        // Arrange
        let (_dir, paths) = setup();
        let event = |name: &str| json!({"hook_event_name": name, "session_id": "s1", "cwd": "/w"});

        // Act & Assert
        assert_eq!(
            run(&paths, &event("SessionStart"), 1),
            Outcome::SessionUpdated(HookStatus::Idle)
        );
        assert_eq!(
            run(&paths, &event("UserPromptSubmit"), 2),
            Outcome::SessionUpdated(HookStatus::Busy)
        );
        assert_eq!(
            run(&paths, &event("PostToolUse"), 3),
            Outcome::SessionUnchanged
        );

        // Assert
        let record: HookSession = read_json(&paths.hook_session_file("s1")).unwrap();
        assert_eq!((record.started_at_ms, record.status_since_ms), (1, 2));

        // Act
        let end_outcome = run(&paths, &event("SessionEnd"), 4);

        // Assert
        assert_eq!(end_outcome, Outcome::SessionRemoved);
        assert!(!paths.hook_session_file("s1").exists());
    }

    #[test]
    fn only_permission_notifications_mean_waiting() {
        // Arrange
        let (_dir, paths) = setup();
        let idle = json!({"hook_event_name": "Notification", "session_id": "s1",
                          "notification_type": "idle_prompt", "message": "Claude is waiting for your input"});

        // Act
        let idle_outcome = run(&paths, &idle, 1);

        // Assert
        assert_eq!(idle_outcome, Outcome::Ignored);

        // Arrange
        let permission_event = json!({"hook_event_name": "Notification", "session_id": "s1",
                          "message": "Claude needs your permission to use Bash"});

        // Act
        let permission_outcome = run(&paths, &permission_event, 2);

        // Assert
        assert_eq!(
            permission_outcome,
            Outcome::SessionUpdated(HookStatus::Waiting)
        );
    }

    #[test]
    fn subagent_lifecycle() {
        // Arrange
        let (_dir, paths) = setup();
        let start = json!({"hook_event_name": "SubagentStart", "session_id": "s1",
                           "agent_id": "a1", "agent_type": "Explore"});

        // Act
        let start_outcome = run(&paths, &start, 10);

        // Assert
        assert_eq!(start_outcome, Outcome::AgentStarted);
        let agent: HookAgent = read_json(&paths.agent_file("s1", "a1")).unwrap();
        assert_eq!(agent.agent_type, "Explore");

        // Arrange
        let stop = json!({"hook_event_name": "SubagentStop", "session_id": "s1", "agent_id": "a1"});

        // Act
        let stop_outcome = run(&paths, &stop, 11);

        // Assert
        assert_eq!(stop_outcome, Outcome::AgentStopped);
        assert!(!paths.agent_file("s1", "a1").exists());
    }

    #[test]
    fn hostile_ids_cannot_escape_the_state_dir() {
        // Arrange
        let (dir, paths) = setup();
        let start = json!({"hook_event_name": "SubagentStart", "session_id": "../../x", "agent_id": "../y"});

        // Act
        run(&paths, &start, 1);

        // Assert
        assert!(paths.agent_file("x", "y").exists());
        assert!(!dir.path().parent().unwrap().join("x").exists());
    }

    #[test]
    fn unknown_events_and_garbage_are_harmless() {
        // Arrange
        let (_dir, paths) = setup();

        // Act & Assert
        assert_eq!(
            run(
                &paths,
                &json!({"hook_event_name": "PreCompact", "session_id": "s"}),
                1
            ),
            Outcome::Ignored
        );
        assert_eq!(
            run(&paths, &json!({"hook_event_name": "Stop"}), 1),
            Outcome::Ignored
        );
        assert!(handle("not json", &paths, 1).is_err());
    }
}
