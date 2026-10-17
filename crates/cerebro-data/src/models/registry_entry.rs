use std::path::{Path, PathBuf};

use cerebro_domain::entities::{Origin, Session, SessionId, SessionStatus};
use cerebro_domain::ports::ObservedSession;
use cerebro_domain::time::Timestamp;
use serde::Deserialize;

/// One `~/.claude/sessions/<pid>.json` file, written by Claude Code itself.
///
/// This is an internal Claude Code format: every field except `pid` and `sessionId`
/// is optional so a format change degrades gracefully instead of hiding sessions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryEntry {
    pub pid: u32,
    pub session_id: String,
    #[serde(default)]
    pub cwd: String,
    pub name: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<u64>,
    pub status_updated_at: Option<u64>,
    pub proc_start: Option<String>,
}

impl From<RegistryEntry> for ObservedSession {
    fn from(entry: RegistryEntry) -> Self {
        let started_at = Timestamp(entry.started_at.unwrap_or_default());
        let cwd = PathBuf::from(&entry.cwd);
        Self {
            session: Session {
                id: SessionId::new(entry.session_id),
                pid: Some(entry.pid),
                name: entry.name.unwrap_or_else(|| folder_name_of(&cwd)),
                status: SessionStatus::parse(entry.status.as_deref().unwrap_or("unknown")),
                status_since: entry.status_updated_at.map_or(started_at, Timestamp),
                started_at,
                cwd,
                origin: Origin::Registry,
                agents: Vec::new(),
            },
            process_start: entry.proc_start,
        }
    }
}

pub(crate) fn folder_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    )
}
