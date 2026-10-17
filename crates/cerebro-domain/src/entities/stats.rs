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
        Agent, Cast, CharacterId, DataMode, Origin, RoomId, Scene, Session, SessionStatus,
    };
    use crate::time::Timestamp;

    fn scene(id: &str, started: u64, agents: &[(&str, &str, u64)]) -> Scene {
        Scene {
            session: Session {
                id: SessionId::new(id),
                pid: Some(1),
                name: id.into(),
                cwd: "/".into(),
                status: SessionStatus::Busy,
                started_at: Timestamp(started),
                status_since: Timestamp(started),
                origin: Origin::Registry,
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
            mode: DataMode::Registry,
            taken_at: Timestamp(1_000),
        }
    }

    #[test]
    #[ignore = "exercise 5"]
    fn empty_snapshot_gives_empty_stats() {
        assert_eq!(compute_stats(&snapshot(vec![])), Stats::default());
    }

    #[test]
    #[ignore = "exercise 5"]
    fn counts_and_records() {
        let stats = compute_stats(&snapshot(vec![
            scene("a", 1, &[("x", "Explore", 400)]),
            scene("b", 2, &[("y", "Explore", 100), ("z", "Plan", 900)]),
        ]));
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
        let stats = compute_stats(&snapshot(vec![
            scene("new", 9, &[("x", "Plan", 1)]),
            scene("old", 1, &[("y", "Plan", 1)]),
        ]));
        assert_eq!(stats.busiest_session, Some(SessionId::new("old")));
    }
}
