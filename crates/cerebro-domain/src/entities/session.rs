use std::path::PathBuf;

use super::{AgentId, SessionId, ToolId};
use crate::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    /// The AI is working on a prompt.
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
    pub tool: ToolId,
    /// Process id of the tool's instance, when known. Without it the session can't be focused.
    pub pid: Option<u32>,
    pub name: String,
    pub cwd: PathBuf,
    pub status: SessionStatus,
    pub started_at: Timestamp,
    pub status_since: Timestamp,
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
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.pad(self.as_str())
    }
}

/// EXERCISE 2: one line for the CLI, e.g. `personal-f4  busy  2 agents  ~/personal`.
impl std::fmt::Display for Session {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let agent_count = self.agents.len();
        let plural = if agent_count == 1 { "" } else { "s" };
        write!(output, "{}", self.name)?;
        write!(output, "  {}", self.status)?;
        write!(output, "  {agent_count} agent{plural}")?;
        write!(output, "  {}", self.cwd.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(agents: usize) -> Session {
        Session {
            id: SessionId::new("s1"),
            tool: ToolId::new("tool-a"),
            pid: Some(1),
            name: "personal-f4".into(),
            cwd: "/Users/me/personal".into(),
            status: SessionStatus::Busy,
            started_at: Timestamp(0),
            status_since: Timestamp(0),
            agents: (0..agents)
                .map(|index| Agent {
                    id: AgentId::new(format!("a{index}")),
                    kind: "Explore".into(),
                    started_at: Timestamp(0),
                })
                .collect(),
        }
    }

    #[test]
    fn status_display() {
        // Act & Assert
        assert_eq!(SessionStatus::Busy.to_string(), "busy");
        assert_eq!(
            SessionStatus::Other("compacting".into()).to_string(),
            "compacting"
        );
    }

    #[test]
    fn session_display_counts_agents() {
        // Act
        let line = session(2).to_string();

        // Assert
        assert!(line.contains("personal-f4"), "{line}");
        assert!(line.contains("busy"), "{line}");
        assert!(line.contains("2 agents"), "{line}");

        // Act
        let singular_line = session(1).to_string();

        // Assert
        assert!(singular_line.contains("1 agent "), "singular, then a space");
    }
}
