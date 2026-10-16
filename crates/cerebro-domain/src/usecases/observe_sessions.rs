use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use crate::DomainError;
use crate::entities::{
    Assignments, Catalog, DataMode, Scene, Session, SessionId, SessionStatus, Snapshot,
};
use crate::ports::{
    AgentSource, AssignmentStore, Clock, ObservedSession, ProcessInspector, SessionSource,
};
use crate::strategies::cast_all;

/// Hook activity this recent, with an empty registry, means the registry is broken.
const FALLBACK_ACTIVITY_WINDOW_MS: u64 = 2 * 60 * 1000;

pub struct ObserveSessionsDeps {
    /// Claude Code's own registry (primary source).
    pub registry: Arc<dyn SessionSource>,
    /// Cerebro hook records (enrichment and fallback).
    pub hooks: Arc<dyn SessionSource>,
    pub agents: Arc<dyn AgentSource>,
    pub processes: Arc<dyn ProcessInspector>,
    pub store: Arc<dyn AssignmentStore>,
    pub clock: Arc<dyn Clock>,
    pub catalog: Catalog,
}

/// Builds the current [`Snapshot`]: live sessions, their subagents and their cast.
pub struct ObserveSessions {
    deps: ObserveSessionsDeps,
    assignments: Mutex<Assignments>,
}

impl ObserveSessions {
    pub fn new(deps: ObserveSessionsDeps) -> Self {
        let assignments = Mutex::new(deps.store.load());
        Self { deps, assignments }
    }

    pub fn execute(&self) -> Result<Snapshot, DomainError> {
        let now = self.deps.clock.now();
        let hook_sessions = self.deps.hooks.observe().unwrap_or_default();

        let (mut sessions, mode) = match self.deps.registry.observe() {
            Err(error) => (
                strip(hook_sessions),
                DataMode::HooksFallback {
                    reason: error.to_string(),
                },
            ),
            Ok(registry) => {
                let live = self.verified(registry);
                let recent_hook_activity = hook_sessions
                    .iter()
                    .any(|h| now.since(h.session.status_since) < FALLBACK_ACTIVITY_WINDOW_MS);
                if live.is_empty() && recent_hook_activity {
                    let reason =
                        "Claude Code's session registry is empty while hooks report activity"
                            .into();
                    (strip(hook_sessions), DataMode::HooksFallback { reason })
                } else {
                    (overlay_waiting(live, &hook_sessions), DataMode::Registry)
                }
            }
        };

        for session in &mut sessions {
            session.agents = self.deps.agents.agents_of(&session.id);
        }
        let alive: Vec<SessionId> = sessions.iter().map(|s| s.id.clone()).collect();
        self.deps.agents.forget_all_except(&alive);

        // Launch order keeps cards in place when a session changes status.
        sessions.sort_by(|a, b| {
            a.started_at
                .cmp(&b.started_at)
                .then_with(|| a.id.cmp(&b.id))
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
        Ok(Snapshot {
            scenes,
            mode,
            taken_at: now,
        })
    }

    /// Keeps registry sessions whose process is alive and is still the same process
    /// (a recycled pid has a different start time: report item m2).
    fn verified(&self, registry: Vec<ObservedSession>) -> Vec<Session> {
        let pids: Vec<u32> = registry.iter().filter_map(|o| o.session.pid).collect();
        let processes = self.deps.processes.inspect(&pids);
        registry
            .into_iter()
            .filter(|observed| {
                let Some(pid) = observed.session.pid else {
                    return false;
                };
                let Some(info) = processes.get(&pid) else {
                    return false;
                };
                match (&observed.process_start, &info.start) {
                    (Some(recorded), Some(actual)) => same_start(recorded, actual),
                    _ => true,
                }
            })
            .map(|observed| observed.session)
            .collect()
    }
}

fn strip(observed: Vec<ObservedSession>) -> Vec<Session> {
    observed.into_iter().map(|o| o.session).collect()
}

/// The registry doesn't know about permission prompts; the hooks do.
fn overlay_waiting(mut sessions: Vec<Session>, hooks: &[ObservedSession]) -> Vec<Session> {
    let by_id: HashMap<&SessionId, &Session> =
        hooks.iter().map(|h| (&h.session.id, &h.session)).collect();
    for session in &mut sessions {
        if let Some(hook) = by_id.get(&session.id)
            && hook.status == SessionStatus::Waiting
            && hook.status_since >= session.status_since
        {
            session.status = SessionStatus::Waiting;
            session.status_since = hook.status_since;
        }
    }
    sessions
}

/// Compares two `ps`-style dates while ignoring whitespace differences.
fn same_start(a: &str, b: &str) -> bool {
    a.split_whitespace().eq(b.split_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Agent, AgentId, CharacterId, Origin, RoomId};
    use crate::ports::ProcessInfo;
    use crate::time::Timestamp;

    struct FakeSource(Result<Vec<ObservedSession>, DomainError>);
    impl SessionSource for FakeSource {
        fn observe(&self) -> Result<Vec<ObservedSession>, DomainError> {
            self.0.clone()
        }
    }

    struct FakeAgents(Vec<(SessionId, Agent)>);
    impl AgentSource for FakeAgents {
        fn agents_of(&self, session: &SessionId) -> Vec<Agent> {
            self.0
                .iter()
                .filter(|(s, _)| s == session)
                .map(|(_, a)| a.clone())
                .collect()
        }
        fn forget_all_except(&self, _alive: &[SessionId]) {}
    }

    struct FakeProcesses(HashMap<u32, ProcessInfo>);
    impl ProcessInspector for FakeProcesses {
        fn inspect(&self, _pids: &[u32]) -> HashMap<u32, ProcessInfo> {
            self.0.clone()
        }
    }

    #[derive(Default)]
    struct MemoryStore(Mutex<Assignments>);
    impl AssignmentStore for MemoryStore {
        fn load(&self) -> Assignments {
            self.0.lock().unwrap().clone()
        }
        fn save(&self, a: &Assignments) -> Result<(), DomainError> {
            *self.0.lock().unwrap() = a.clone();
            Ok(())
        }
    }

    struct FixedClock(u64);
    impl Clock for FixedClock {
        fn now(&self) -> Timestamp {
            Timestamp(self.0)
        }
    }

    fn observed(
        id: &str,
        pid: Option<u32>,
        start: Option<&str>,
        status: SessionStatus,
        since: u64,
    ) -> ObservedSession {
        ObservedSession {
            session: Session {
                id: SessionId::new(id),
                pid,
                name: id.into(),
                cwd: "/w".into(),
                status,
                started_at: Timestamp(since),
                status_since: Timestamp(since),
                origin: if pid.is_some() {
                    Origin::Registry
                } else {
                    Origin::Hooks
                },
                agents: vec![],
            },
            process_start: start.map(Into::into),
        }
    }

    fn alive(entries: &[(u32, Option<&str>)]) -> FakeProcesses {
        FakeProcesses(
            entries
                .iter()
                .map(|(p, s)| {
                    (
                        *p,
                        ProcessInfo {
                            start: s.map(Into::into),
                        },
                    )
                })
                .collect(),
        )
    }

    fn use_case(
        registry: Result<Vec<ObservedSession>, DomainError>,
        hooks: Vec<ObservedSession>,
        processes: FakeProcesses,
        agents: FakeAgents,
        now: u64,
    ) -> ObserveSessions {
        ObserveSessions::new(ObserveSessionsDeps {
            registry: Arc::new(FakeSource(registry)),
            hooks: Arc::new(FakeSource(Ok(hooks))),
            agents: Arc::new(agents),
            processes: Arc::new(processes),
            store: Arc::new(MemoryStore::default()),
            clock: Arc::new(FixedClock(now)),
            catalog: Catalog::new(
                vec![CharacterId::new("wolverine"), CharacterId::new("storm")],
                vec![RoomId::new("garage")],
            )
            .unwrap(),
        })
    }

    #[test]
    fn keeps_live_sessions_and_attaches_agents() {
        let agent = Agent {
            id: AgentId::new("a1"),
            kind: "Explore".into(),
            started_at: Timestamp(1),
        };
        let snapshot = use_case(
            Ok(vec![observed("s1", Some(10), None, SessionStatus::Busy, 1)]),
            vec![],
            alive(&[(10, None)]),
            FakeAgents(vec![(SessionId::new("s1"), agent)]),
            100,
        )
        .execute()
        .unwrap();
        assert_eq!(snapshot.mode, DataMode::Registry);
        assert_eq!(snapshot.character_count(), 2);
        assert_eq!(snapshot.scenes[0].cast.agents.len(), 1);
    }

    #[test]
    fn drops_dead_and_recycled_pids() {
        let snapshot = use_case(
            Ok(vec![
                observed("dead", Some(1), None, SessionStatus::Idle, 1),
                observed(
                    "recycled",
                    Some(2),
                    Some("Thu Oct 15 08:43:57 2026"),
                    SessionStatus::Idle,
                    1,
                ),
                observed(
                    "same",
                    Some(3),
                    Some("Thu Oct 15  9:00:00 2026"),
                    SessionStatus::Idle,
                    1,
                ),
            ]),
            vec![],
            alive(&[
                (2, Some("Fri Oct 16 10:00:00 2026")),
                (3, Some("Thu Oct 15 9:00:00 2026")),
            ]),
            FakeAgents(vec![]),
            100,
        )
        .execute()
        .unwrap();
        let ids: Vec<_> = snapshot
            .scenes
            .iter()
            .map(|s| s.session.id.as_str())
            .collect();
        assert_eq!(ids, ["same"]);
    }

    #[test]
    fn waiting_from_hooks_overrides_an_older_registry_status() {
        let snapshot = use_case(
            Ok(vec![observed("s1", Some(10), None, SessionStatus::Busy, 5)]),
            vec![observed("s1", None, None, SessionStatus::Waiting, 7)],
            alive(&[(10, None)]),
            FakeAgents(vec![]),
            100,
        )
        .execute()
        .unwrap();
        assert_eq!(snapshot.scenes[0].session.status, SessionStatus::Waiting);
    }

    #[test]
    fn falls_back_to_hooks_when_registry_fails() {
        let snapshot = use_case(
            Err(DomainError::SourceUnavailable("gone".into())),
            vec![observed("h1", None, None, SessionStatus::Busy, 1)],
            alive(&[]),
            FakeAgents(vec![]),
            100,
        )
        .execute()
        .unwrap();
        assert!(matches!(snapshot.mode, DataMode::HooksFallback { .. }));
        assert_eq!(snapshot.scenes.len(), 1);
    }

    #[test]
    fn falls_back_when_registry_is_empty_but_hooks_are_active() {
        let now = 10 * 60 * 1000;
        let recent = use_case(
            Ok(vec![]),
            vec![observed("h1", None, None, SessionStatus::Busy, now - 1000)],
            alive(&[]),
            FakeAgents(vec![]),
            now,
        )
        .execute()
        .unwrap();
        assert!(matches!(recent.mode, DataMode::HooksFallback { .. }));

        // Stale hook records (e.g. a crashed session) must not trigger the fallback.
        let stale = use_case(
            Ok(vec![]),
            vec![observed("h1", None, None, SessionStatus::Busy, 0)],
            alive(&[]),
            FakeAgents(vec![]),
            now,
        )
        .execute()
        .unwrap();
        assert_eq!(stale.mode, DataMode::Registry);
        assert_eq!(stale.scenes, []);
    }

    #[test]
    fn sessions_are_ordered_by_launch_time() {
        let snapshot = use_case(
            Ok(vec![
                observed("late", Some(1), None, SessionStatus::Busy, 9),
                observed("early", Some(2), None, SessionStatus::Idle, 3),
            ]),
            vec![],
            alive(&[(1, None), (2, None)]),
            FakeAgents(vec![]),
            100,
        )
        .execute()
        .unwrap();
        let ids: Vec<_> = snapshot
            .scenes
            .iter()
            .map(|s| s.session.id.as_str())
            .collect();
        assert_eq!(ids, ["early", "late"]);
    }
}
