use std::collections::BTreeMap;

use super::{AgentId, SessionId, Snapshot};

/// Numbers for `cerebro stats` (EXERCISE 5).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    /// How many running subagents of each type, e.g. `{"Explore": 3, "Plan": 1}`.
    pub agents_by_kind: BTreeMap<String, usize>,
    /// The session with the most subagents (oldest session wins a tie).
    pub busiest_session: Option<SessionId>,
    /// The subagent that started first, with its age in milliseconds at `snapshot.taken_at`.
    pub longest_running_agent: Option<(AgentId, u64)>,
}

/// EXERCISE 5: compute [`Stats`] with iterators only (no `for` loops, no `mut` locals).
pub fn compute_stats(snapshot: &Snapshot) -> Stats {
    todo!("Exercise 5: stats for {} scenes", snapshot.scenes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        Agent, Cast, CharacterId, RoomId, Scene, Session, SessionStatus, ToolId,
    };
    use crate::time::Timestamp;

    fn scene(id: &str, started: u64, agents: &[(&str, &str, u64)]) -> Scene {
        Scene {
            session: Session {
                id: SessionId::new(id),
                tool: ToolId::new("tool-a"),
                pid: Some(1),
                name: id.into(),
                cwd: "/".into(),
                status: SessionStatus::Busy,
                started_at: Timestamp(started),
                status_since: Timestamp(started),
                remote_url: None,
                agents: agents
                    .iter()
                    .map(|(agent_id, kind, agent_started)| Agent {
                        id: AgentId::new(*agent_id),
                        kind: (*kind).into(),
                        started_at: Timestamp(*agent_started),
                    })
                    .collect(),
            },
            cast: Cast {
                room: RoomId::new("r"),
                lead: CharacterId::new("c"),
                agents: vec![],
            },
        }
    }

    fn snapshot(scenes: Vec<Scene>) -> Snapshot {
        Snapshot {
            scenes,
            warnings: vec![],
            taken_at: Timestamp(1_000),
        }
    }

    #[test]
    #[ignore = "exercise 5"]
    fn empty_snapshot_gives_empty_stats() {
        // Arrange
        let empty = snapshot(vec![]);

        // Act
        let stats = compute_stats(&empty);

        // Assert
        assert_eq!(stats, Stats::default());
    }

    #[test]
    #[ignore = "exercise 5"]
    fn counts_and_records() {
        // Arrange
        let observed = snapshot(vec![
            scene("a", 1, &[("x", "Explore", 400)]),
            scene("b", 2, &[("y", "Explore", 100), ("z", "Plan", 900)]),
        ]);

        // Act
        let stats = compute_stats(&observed);

        // Assert
        assert_eq!(
            stats.agents_by_kind,
            BTreeMap::from([("Explore".into(), 2), ("Plan".into(), 1)])
        );
        assert_eq!(stats.busiest_session, Some(SessionId::new("b")));
        assert_eq!(stats.longest_running_agent, Some((AgentId::new("y"), 900)));
    }

    #[test]
    #[ignore = "exercise 5"]
    fn tie_goes_to_the_oldest_session() {
        // Arrange
        let observed = snapshot(vec![
            scene("new", 9, &[("x", "Plan", 1)]),
            scene("old", 1, &[("y", "Plan", 1)]),
        ]);

        // Act
        let stats = compute_stats(&observed);

        // Assert
        assert_eq!(stats.busiest_session, Some(SessionId::new("old")));
    }
}
