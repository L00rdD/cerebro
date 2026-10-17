use std::sync::{Arc, Mutex, PoisonError};

use crate::entities::{Assignments, Catalog, Scene, Snapshot, SourceWarning};
use crate::ports::{AssignmentStore, Clock, SessionSource};
use crate::strategies::cast_all;

pub struct ObserveSessionsDeps {
    /// One source per AI tool. A failing source doesn't hide the others' sessions.
    pub sources: Vec<Arc<dyn SessionSource>>,
    pub store: Arc<dyn AssignmentStore>,
    pub clock: Arc<dyn Clock>,
    pub catalog: Catalog,
}

/// Builds the current [`Snapshot`]: live sessions of every source and their cast.
pub struct ObserveSessions {
    deps: ObserveSessionsDeps,
    assignments: Mutex<Assignments>,
}

impl ObserveSessions {
    pub fn new(deps: ObserveSessionsDeps) -> Self {
        let assignments = Mutex::new(deps.store.load());
        Self { deps, assignments }
    }

    pub fn execute(&self) -> Snapshot {
        let now = self.deps.clock.now();
        let mut sessions = Vec::new();
        let mut warnings = Vec::new();
        for source in &self.deps.sources {
            match source.observe() {
                Ok(observation) => {
                    sessions.extend(observation.sessions);
                    if let Some(message) = observation.warning {
                        warnings.push(SourceWarning {
                            tool: source.tool(),
                            message,
                        });
                    }
                }
                Err(error) => warnings.push(SourceWarning {
                    tool: source.tool(),
                    message: error.to_string(),
                }),
            }
        }

        // Launch order keeps cards in place when a session changes status.
        sessions.sort_by(|left, right| {
            left.started_at
                .cmp(&right.started_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        let mut assignments = self
            .assignments
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let (casts, changed) = cast_all(&self.deps.catalog, &mut assignments, &sessions);
        if changed {
            // Not fatal: picks stay stable in memory even if the file can't be written.
            let _ = self.deps.store.save(&assignments);
        }

        let scenes = sessions
            .into_iter()
            .zip(casts)
            .map(|(session, cast)| Scene { session, cast })
            .collect();
        Snapshot {
            scenes,
            warnings,
            taken_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainError;
    use crate::entities::{
        Agent, AgentId, CharacterId, RoomId, Session, SessionId, SessionStatus, ToolId,
    };
    use crate::ports::Observation;
    use crate::time::Timestamp;

    struct FakeSource {
        tool: &'static str,
        result: Result<Observation, DomainError>,
    }
    impl SessionSource for FakeSource {
        fn tool(&self) -> ToolId {
            ToolId::new(self.tool)
        }
        fn observe(&self) -> Result<Observation, DomainError> {
            self.result.clone()
        }
    }

    #[derive(Default)]
    struct MemoryStore(Mutex<Assignments>);
    impl AssignmentStore for MemoryStore {
        fn load(&self) -> Assignments {
            self.0.lock().unwrap().clone()
        }
        fn save(&self, assignments: &Assignments) -> Result<(), DomainError> {
            *self.0.lock().unwrap() = assignments.clone();
            Ok(())
        }
    }

    struct FixedClock(u64);
    impl Clock for FixedClock {
        fn now(&self) -> Timestamp {
            Timestamp(self.0)
        }
    }

    fn session(tool: &str, id: &str, started: u64, agents: usize) -> Session {
        Session {
            id: SessionId::new(id),
            tool: ToolId::new(tool),
            pid: Some(1),
            name: id.into(),
            cwd: "/w".into(),
            status: SessionStatus::Busy,
            started_at: Timestamp(started),
            status_since: Timestamp(started),
            agents: (0..agents)
                .map(|index| Agent {
                    id: AgentId::new(format!("{id}-a{index}")),
                    kind: "Explore".into(),
                    started_at: Timestamp(started),
                })
                .collect(),
        }
    }

    fn source(tool: &'static str, sessions: Vec<Session>, warning: Option<&str>) -> FakeSource {
        FakeSource {
            tool,
            result: Ok(Observation {
                sessions,
                warning: warning.map(Into::into),
            }),
        }
    }

    fn use_case(sources: Vec<FakeSource>) -> ObserveSessions {
        ObserveSessions::new(ObserveSessionsDeps {
            sources: sources
                .into_iter()
                .map(|source| Arc::new(source) as Arc<dyn SessionSource>)
                .collect(),
            store: Arc::new(MemoryStore::default()),
            clock: Arc::new(FixedClock(100)),
            catalog: Catalog::new(
                vec![CharacterId::new("wolverine"), CharacterId::new("storm")],
                vec![RoomId::new("garage")],
            )
            .unwrap(),
        })
    }

    fn ids(snapshot: &Snapshot) -> Vec<&str> {
        snapshot
            .scenes
            .iter()
            .map(|scene| scene.session.id.as_str())
            .collect()
    }

    #[test]
    fn casts_every_session_and_agent() {
        let snapshot = use_case(vec![source(
            "tool-a",
            vec![session("tool-a", "s1", 1, 1)],
            None,
        )])
        .execute();
        assert_eq!(snapshot.warnings, []);
        assert_eq!(snapshot.character_count(), 2);
        assert_eq!(snapshot.scenes[0].cast.agents.len(), 1);
        assert_eq!(snapshot.taken_at, Timestamp(100));
    }

    #[test]
    fn merges_sources_in_launch_order() {
        let snapshot = use_case(vec![
            source("tool-a", vec![session("tool-a", "late", 9, 0)], None),
            source("tool-b", vec![session("tool-b", "early", 3, 0)], None),
        ])
        .execute();
        assert_eq!(ids(&snapshot), ["early", "late"]);
    }

    #[test]
    fn a_failing_source_becomes_a_warning_without_hiding_the_others() {
        let snapshot = use_case(vec![
            FakeSource {
                tool: "tool-a",
                result: Err(DomainError::SourceUnavailable("gone".into())),
            },
            source("tool-b", vec![session("tool-b", "s1", 1, 0)], None),
        ])
        .execute();
        assert_eq!(ids(&snapshot), ["s1"]);
        assert_eq!(
            snapshot.warnings,
            [SourceWarning {
                tool: ToolId::new("tool-a"),
                message: "data source unavailable: gone".into(),
            }]
        );
    }

    #[test]
    fn a_degraded_source_keeps_its_sessions_and_reports_why() {
        let snapshot = use_case(vec![source(
            "tool-a",
            vec![session("tool-a", "s1", 1, 0)],
            Some("fallback mode"),
        )])
        .execute();
        assert_eq!(ids(&snapshot), ["s1"]);
        assert_eq!(snapshot.warnings[0].tool, ToolId::new("tool-a"));
        assert_eq!(snapshot.warnings[0].message, "fallback mode");
    }
}
