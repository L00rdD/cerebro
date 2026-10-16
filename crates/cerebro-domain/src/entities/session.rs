use std::path::PathBuf;

use super::{AgentId, SessionId};
use crate::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    /// Claude is working on a prompt.
    Busy,
    /// Waiting for the user's next prompt.
    Idle,
    /// Blocked on a permission prompt.
    Waiting,
    /// A status this version of Cerebro does not know yet.
    Other(String),
}

impl SessionStatus {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "busy" => Self::Busy,
            "idle" => Self::Idle,
            "waiting" => Self::Waiting,
            other => Self::Other(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Busy => "busy",
            Self::Idle => "idle",
            Self::Waiting => "waiting",
            Self::Other(raw) => raw,
        }
    }
}

/// Where a session's data came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Claude Code's own session registry (rich: pid, name, status).
    Registry,
    /// Cerebro's hook records (fallback: no pid, no name).
    Hooks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub id: AgentId,
    /// Subagent type, e.g. `Explore` or `general-purpose`.
    pub kind: String,
    pub started_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: SessionId,
    /// Process id of the Claude Code instance, when known.
    pub pid: Option<u32>,
    pub name: String,
    pub cwd: PathBuf,
    pub status: SessionStatus,
    pub started_at: Timestamp,
    pub status_since: Timestamp,
    pub origin: Origin,
    pub agents: Vec<Agent>,
}

impl Session {
    pub fn is_busy(&self) -> bool {
        self.status == SessionStatus::Busy
    }

    /// One character for the session itself plus one per subagent.
    pub fn character_count(&self) -> usize {
        1 + self.agents.len()
    }
}

/// EXERCISE 2: `busy`, `idle`, `waiting`, or the raw text for unknown statuses.
impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f;
        todo!("Exercise 2: write the status")
    }
}

/// EXERCISE 2: one line for the CLI, e.g. `personal-f4  busy  2 agents  ~/personal`.
impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f;
        todo!("Exercise 2: describe session {}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(agents: usize) -> Session {
        Session {
            id: SessionId::new("s1"),
            pid: Some(1),
            name: "personal-f4".into(),
            cwd: "/Users/me/personal".into(),
            status: SessionStatus::Busy,
            started_at: Timestamp(0),
            status_since: Timestamp(0),
            origin: Origin::Registry,
            agents: (0..agents)
                .map(|i| Agent { id: AgentId::new(format!("a{i}")), kind: "Explore".into(), started_at: Timestamp(0) })
                .collect(),
        }
    }

    #[test]
    #[ignore = "exercise 2"]
    fn status_display() {
        assert_eq!(SessionStatus::Busy.to_string(), "busy");
        assert_eq!(SessionStatus::Other("compacting".into()).to_string(), "compacting");
    }

    #[test]
    #[ignore = "exercise 2"]
    fn session_display_counts_agents() {
        let line = session(2).to_string();
        assert!(line.contains("personal-f4"), "{line}");
        assert!(line.contains("busy"), "{line}");
        assert!(line.contains("2 agents"), "{line}");
        assert!(session(1).to_string().contains("1 agent "), "singular, then a space");
    }
}
