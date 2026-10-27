use std::fs;
use std::path::PathBuf;

use cerebro_domain::DomainError;
use cerebro_domain::entities::{Session, SessionId, SessionStatus, ToolId};
use cerebro_domain::time::Timestamp;
use cerebro_protocol::{HookSession, HookStatus, now_ms, read_json};

use super::{CLAUDE_CODE, ObservedSession, SessionReader};
use crate::models::folder_name_of;

/// Records older than this belong to sessions that died without a `SessionEnd`.
const STALE_AFTER_MS: u64 = 12 * 3600 * 1000;

/// Reads the session records written by `cerebro-hook`.
pub struct HookSessionSource {
    dir: PathBuf,
}

impl HookSessionSource {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}

impl SessionReader for HookSessionSource {
    fn observe(&self) -> Result<Vec<ObservedSession>, DomainError> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Ok(Vec::new()); // Hooks not installed yet: nothing to report.
        };
        let now = now_ms();
        Ok(entries
            .filter_map(Result::ok)
            .filter_map(|entry| read_json::<HookSession>(&entry.path()).ok())
            .filter(|record| now.saturating_sub(record.status_since_ms) < STALE_AFTER_MS)
            .map(|record| ObservedSession {
                session: to_session(record),
                process_start: None,
            })
            .collect())
    }
}

fn to_session(record: HookSession) -> Session {
    let cwd = PathBuf::from(&record.cwd);
    Session {
        id: SessionId::new(record.session_id),
        tool: ToolId::new(CLAUDE_CODE),
        pid: None,
        name: folder_name_of(&cwd),
        cwd,
        status: match record.status {
            HookStatus::Busy => SessionStatus::Busy,
            HookStatus::Idle => SessionStatus::Idle,
            HookStatus::Waiting => SessionStatus::Waiting,
        },
        started_at: Timestamp(record.started_at_ms),
        status_since: Timestamp(record.status_since_ms),
        remote_url: None,
        agents: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerebro_protocol::{FORMAT_VERSION, write_json_atomic};

    fn record(id: &str, since: u64) -> HookSession {
        HookSession {
            version: FORMAT_VERSION,
            session_id: id.into(),
            cwd: "/work/cerebro".into(),
            status: HookStatus::Waiting,
            started_at_ms: since,
            status_since_ms: since,
        }
    }

    #[test]
    fn reads_fresh_records_only() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        write_json_atomic(&dir.path().join("fresh.json"), &record("fresh", now_ms())).unwrap();
        write_json_atomic(&dir.path().join("stale.json"), &record("stale", 1)).unwrap();

        // Act
        let sessions = HookSessionSource::new(dir.path()).observe().unwrap();

        // Assert
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session.name, "cerebro");
        assert_eq!(sessions[0].session.status, SessionStatus::Waiting);
        assert_eq!(sessions[0].session.pid, None);
    }

    #[test]
    fn missing_directory_means_no_sessions() {
        // Act
        let sessions = HookSessionSource::new("/nope").observe().unwrap();

        // Assert
        assert_eq!(sessions, [] as [ObservedSession; 0]);
    }
}
