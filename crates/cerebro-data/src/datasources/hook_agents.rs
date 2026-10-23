use std::fs;
use std::time::{Duration, SystemTime};

use cerebro_domain::entities::{Agent, AgentId, SessionId};
use cerebro_domain::time::Timestamp;
use cerebro_protocol::{HookAgent, Paths, now_ms, read_json, sanitize_id};

use super::AgentReader;

/// Agent files older than this are leftovers of a missed `SubagentStop`.
const AGENT_TTL_MS: u64 = 6 * 3600 * 1000;
/// A session may start an agent slightly before it shows up in the registry.
const ORPHAN_GRACE: Duration = Duration::from_secs(600);

/// Reads the subagent records written by `cerebro-hook`.
pub struct HookAgentSource {
    paths: Paths,
}

impl HookAgentSource {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }
}

impl AgentReader for HookAgentSource {
    fn agents_of(&self, session: &SessionId) -> Vec<Agent> {
        let Some(id) = sanitize_id(session.as_str()) else {
            return Vec::new();
        };
        let Ok(entries) = fs::read_dir(self.paths.session_agents_dir(&id)) else {
            return Vec::new();
        };
        let now = now_ms();
        let mut agents: Vec<Agent> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| read_json::<HookAgent>(&entry.path()).ok())
            .filter(|record| now.saturating_sub(record.started_at_ms) < AGENT_TTL_MS)
            .map(|record| Agent {
                id: AgentId::new(record.agent_id),
                kind: record.agent_type,
                started_at: Timestamp(record.started_at_ms),
            })
            .collect();
        agents.sort_by(|left, right| {
            left.started_at
                .cmp(&right.started_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        agents
    }

    fn forget_all_except(&self, alive: &[SessionId]) {
        let Ok(entries) = fs::read_dir(self.paths.agents_dir()) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if alive.iter().any(|id| id.as_str() == name) {
                continue;
            }
            let age = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .unwrap_or_default();
            if age > ORPHAN_GRACE {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerebro_protocol::{FORMAT_VERSION, write_json_atomic};

    #[test]
    fn lists_agents_oldest_first() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path());
        for (id, kind, started_at) in [("b", "Plan", now_ms()), ("a", "Explore", now_ms() - 5000)] {
            let agent = HookAgent {
                version: FORMAT_VERSION,
                agent_id: id.into(),
                agent_type: kind.into(),
                started_at_ms: started_at,
            };
            write_json_atomic(&paths.agent_file("s1", id), &agent).unwrap();
        }

        // Act
        let agents = HookAgentSource::new(paths).agents_of(&SessionId::new("s1"));

        // Assert
        let kinds: Vec<_> = agents.iter().map(|agent| agent.kind.as_str()).collect();
        assert_eq!(kinds, ["Explore", "Plan"]);
    }

    #[test]
    fn keeps_recent_orphans_during_grace_period() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path());
        fs::create_dir_all(paths.session_agents_dir("young")).unwrap();

        // Act
        HookAgentSource::new(paths.clone()).forget_all_except(&[]);

        // Assert
        assert!(paths.session_agents_dir("young").exists());
    }
}
