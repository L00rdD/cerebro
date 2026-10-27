//! Claude Code as a session source: everything specific to how Claude Code exposes its sessions.
//!
//! Claude Code's registry is the primary source (names, pids, sessions opened before Cerebro).
//! Cerebro's hook records add the *waiting* status and take over when the registry is unreadable,
//! or empty while hooks show recent activity.

use std::collections::HashMap;
use std::sync::Arc;

use cerebro_domain::DomainError;
use cerebro_domain::entities::{Agent, Session, SessionId, SessionStatus, ToolId};
use cerebro_domain::ports::{Clock, Observation, SessionSource};

use crate::services::ProcessInspector;

/// [`ToolId`] of every session read from Claude Code.
pub const CLAUDE_CODE: &str = "claude-code";

/// Hook activity this recent, with an empty registry, means the registry is broken.
const FALLBACK_ACTIVITY_WINDOW_MS: u64 = 2 * 60 * 1000;

/// A session as read from a file, before process verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedSession {
    pub session: Session,
    /// Process start time as recorded by the source (e.g. `"Thu Oct 15 08:43:57 2026"`),
    /// used to detect a recycled pid.
    pub process_start: Option<String>,
}

/// Reads raw session records (Claude Code's registry, Cerebro's hook records).
pub trait SessionReader: Send + Sync {
    fn observe(&self) -> Result<Vec<ObservedSession>, DomainError>;
}

/// Running subagents, recorded by the hook.
pub trait AgentReader: Send + Sync {
    fn agents_of(&self, session: &SessionId) -> Vec<Agent>;
    /// Drops leftovers of sessions that are gone without a `SessionEnd` event.
    fn forget_all_except(&self, alive: &[SessionId]);
}

pub struct ClaudeCodeDeps {
    /// Claude Code's own registry (primary source).
    pub registry: Arc<dyn SessionReader>,
    /// Cerebro hook records (enrichment and fallback).
    pub hooks: Arc<dyn SessionReader>,
    pub agents: Arc<dyn AgentReader>,
    pub processes: Arc<dyn ProcessInspector>,
    pub clock: Arc<dyn Clock>,
}

pub struct ClaudeCodeSource {
    deps: ClaudeCodeDeps,
}

impl ClaudeCodeSource {
    pub fn new(deps: ClaudeCodeDeps) -> Self {
        Self { deps }
    }

    /// Keeps registry sessions whose process is alive and is still the same process
    /// (a recycled pid has a different start time: report item m2).
    fn verified(&self, registry: Vec<ObservedSession>) -> Vec<Session> {
        let pids: Vec<u32> = registry
            .iter()
            .filter_map(|observed| observed.session.pid)
            .collect();
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

impl SessionSource for ClaudeCodeSource {
    fn tool(&self) -> ToolId {
        ToolId::new(CLAUDE_CODE)
    }

    fn observe(&self) -> Result<Observation, DomainError> {
        let now = self.deps.clock.now();
        let hook_sessions = self.deps.hooks.observe().unwrap_or_default();

        let (mut sessions, warning) = match self.deps.registry.observe() {
            Err(error) => (strip(hook_sessions), Some(fallback(&error.to_string()))),
            Ok(registry) => {
                let live = self.verified(registry);
                let recent_hook_activity = hook_sessions
                    .iter()
                    .any(|hook| now.since(hook.session.status_since) < FALLBACK_ACTIVITY_WINDOW_MS);
                if live.is_empty() && recent_hook_activity {
                    let reason = fallback(
                        "Claude Code's session registry is empty while hooks report activity",
                    );
                    (strip(hook_sessions), Some(reason))
                } else {
                    (overlay_waiting(live, &hook_sessions), None)
                }
            }
        };

        for session in &mut sessions {
            session.agents = self.deps.agents.agents_of(&session.id);
        }
        let alive: Vec<SessionId> = sessions.iter().map(|session| session.id.clone()).collect();
        self.deps.agents.forget_all_except(&alive);

        Ok(Observation { sessions, warning })
    }
}

fn fallback(reason: &str) -> String {
    format!(
        "Fallback mode: {reason}. Sessions come from Cerebro's hooks only \
         (no names, no click-to-focus)."
    )
}

fn strip(observed: Vec<ObservedSession>) -> Vec<Session> {
    observed
        .into_iter()
        .map(|observed_session| observed_session.session)
        .collect()
}

/// The registry doesn't know about permission prompts; the hooks do.
fn overlay_waiting(mut sessions: Vec<Session>, hooks: &[ObservedSession]) -> Vec<Session> {
    let by_id: HashMap<&SessionId, &Session> = hooks
        .iter()
        .map(|hook| (&hook.session.id, &hook.session))
        .collect();
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
fn same_start(recorded: &str, actual: &str) -> bool {
    recorded.split_whitespace().eq(actual.split_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ProcessInfo;
    use cerebro_domain::entities::AgentId;
    use cerebro_domain::time::Timestamp;

    struct FakeReader(Result<Vec<ObservedSession>, DomainError>);
    impl SessionReader for FakeReader {
        fn observe(&self) -> Result<Vec<ObservedSession>, DomainError> {
            self.0.clone()
        }
    }

    struct FakeAgents(Vec<(SessionId, Agent)>);
    impl AgentReader for FakeAgents {
        fn agents_of(&self, session: &SessionId) -> Vec<Agent> {
            self.0
                .iter()
                .filter(|(owner, _)| owner == session)
                .map(|(_, agent)| agent.clone())
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
                tool: ToolId::new(CLAUDE_CODE),
                pid,
                name: id.into(),
                cwd: "/w".into(),
                status,
                started_at: Timestamp(since),
                status_since: Timestamp(since),
                remote_url: None,
                agents: vec![],
            },
            process_start: start.map(Into::into),
        }
    }

    fn alive(entries: &[(u32, Option<&str>)]) -> FakeProcesses {
        FakeProcesses(
            entries
                .iter()
                .map(|(pid, start)| {
                    (
                        *pid,
                        ProcessInfo {
                            start: start.map(Into::into),
                        },
                    )
                })
                .collect(),
        )
    }

    fn source(
        registry: Result<Vec<ObservedSession>, DomainError>,
        hooks: Vec<ObservedSession>,
        processes: FakeProcesses,
        agents: FakeAgents,
        now: u64,
    ) -> ClaudeCodeSource {
        ClaudeCodeSource::new(ClaudeCodeDeps {
            registry: Arc::new(FakeReader(registry)),
            hooks: Arc::new(FakeReader(Ok(hooks))),
            agents: Arc::new(agents),
            processes: Arc::new(processes),
            clock: Arc::new(FixedClock(now)),
        })
    }

    fn ids(observation: &Observation) -> Vec<&str> {
        observation
            .sessions
            .iter()
            .map(|session| session.id.as_str())
            .collect()
    }

    #[test]
    fn keeps_live_sessions_and_attaches_agents() {
        // Arrange
        let agent = Agent {
            id: AgentId::new("a1"),
            kind: "Explore".into(),
            started_at: Timestamp(1),
        };

        // Act
        let observation = source(
            Ok(vec![observed("s1", Some(10), None, SessionStatus::Busy, 1)]),
            vec![],
            alive(&[(10, None)]),
            FakeAgents(vec![(SessionId::new("s1"), agent)]),
            100,
        )
        .observe()
        .unwrap();

        // Assert
        assert_eq!(observation.warning, None);
        assert_eq!(observation.sessions[0].agents.len(), 1);
        assert_eq!(observation.sessions[0].tool, ToolId::new(CLAUDE_CODE));
    }

    #[test]
    fn drops_dead_and_recycled_pids() {
        // Act
        let observation = source(
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
        .observe()
        .unwrap();

        // Assert
        assert_eq!(ids(&observation), ["same"]);
    }

    #[test]
    fn waiting_from_hooks_overrides_an_older_registry_status() {
        // Act
        let observation = source(
            Ok(vec![observed("s1", Some(10), None, SessionStatus::Busy, 5)]),
            vec![observed("s1", None, None, SessionStatus::Waiting, 7)],
            alive(&[(10, None)]),
            FakeAgents(vec![]),
            100,
        )
        .observe()
        .unwrap();

        // Assert
        assert_eq!(observation.sessions[0].status, SessionStatus::Waiting);
    }

    #[test]
    fn falls_back_to_hooks_when_registry_fails() {
        // Act
        let observation = source(
            Err(DomainError::SourceUnavailable("gone".into())),
            vec![observed("h1", None, None, SessionStatus::Busy, 1)],
            alive(&[]),
            FakeAgents(vec![]),
            100,
        )
        .observe()
        .unwrap();

        // Assert
        assert!(
            observation.warning.as_deref().is_some_and(
                |warning| warning.starts_with("Fallback mode: data source unavailable")
            )
        );
        assert_eq!(ids(&observation), ["h1"]);
    }

    #[test]
    fn falls_back_when_registry_is_empty_but_hooks_are_active() {
        // Arrange
        let now = 10 * 60 * 1000;

        // Act
        let recent = source(
            Ok(vec![]),
            vec![observed("h1", None, None, SessionStatus::Busy, now - 1000)],
            alive(&[]),
            FakeAgents(vec![]),
            now,
        )
        .observe()
        .unwrap();

        // Assert
        assert!(recent.warning.is_some());
        assert_eq!(ids(&recent), ["h1"]);

        // Stale hook records (e.g. a crashed session) must not trigger the fallback.
        // Act
        let stale = source(
            Ok(vec![]),
            vec![observed("h1", None, None, SessionStatus::Busy, 0)],
            alive(&[]),
            FakeAgents(vec![]),
            now,
        )
        .observe()
        .unwrap();

        // Assert
        assert_eq!(stale, Observation::default());
    }
}
