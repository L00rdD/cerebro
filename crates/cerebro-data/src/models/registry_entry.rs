use std::path::{Path, PathBuf};

use cerebro_domain::entities::{Session, SessionId, SessionStatus, ToolId};
use cerebro_domain::time::Timestamp;
use serde::Deserialize;

use crate::datasources::{CLAUDE_CODE, ObservedSession};

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
    /// Set while Remote Control is on: the session can be continued from claude.ai or the app.
    pub bridge_session_id: Option<String>,
}

impl From<RegistryEntry> for ObservedSession {
    fn from(entry: RegistryEntry) -> Self {
        let started_at = Timestamp(entry.started_at.unwrap_or_default());
        let cwd = PathBuf::from(&entry.cwd);
        Self {
            session: Session {
                id: SessionId::new(entry.session_id),
                tool: ToolId::new(CLAUDE_CODE),
                pid: Some(entry.pid),
                name: entry.name.unwrap_or_else(|| folder_name_of(&cwd)),
                status: SessionStatus::parse(entry.status.as_deref().unwrap_or("unknown")),
                status_since: entry.status_updated_at.map_or(started_at, Timestamp),
                remote_url: entry
                    .bridge_session_id
                    .as_deref()
                    .and_then(remote_control_url),
                started_at,
                cwd,
                agents: Vec::new(),
            },
            process_start: entry.proc_start,
        }
    }
}

/// Claude Code's own link to a Remote Control session: its `cse_` bridge id is shown as `session_`.
/// Ids with anything but `[A-Za-z0-9_-]` are rejected, so the link can't point elsewhere.
fn remote_control_url(bridge_session_id: &str) -> Option<String> {
    let safe = !bridge_session_id.is_empty()
        && bridge_session_id
            .chars()
            .all(|symbol| symbol.is_ascii_alphanumeric() || symbol == '_' || symbol == '-');
    if !safe {
        return None;
    }
    let id = bridge_session_id.strip_prefix("cse_").map_or_else(
        || bridge_session_id.to_owned(),
        |body| format!("session_{body}"),
    );
    Some(format!("https://claude.ai/code/{id}"))
}

pub(crate) fn folder_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_control_links_to_the_session_on_claude_ai() {
        // Act & Assert
        assert_eq!(
            remote_control_url("cse_01AbC-d_e").as_deref(),
            Some("https://claude.ai/code/session_01AbC-d_e")
        );
        assert_eq!(
            remote_control_url("session_01AbC").as_deref(),
            Some("https://claude.ai/code/session_01AbC")
        );
    }

    #[test]
    fn remote_control_rejects_unsafe_ids() {
        // Act & Assert
        assert_eq!(remote_control_url(""), None);
        assert_eq!(remote_control_url("cse_../../evil"), None);
        assert_eq!(remote_control_url("cse_a?b=c"), None);
    }

    #[test]
    fn remote_control_is_off_without_a_bridge_id() {
        // Arrange
        let entry: RegistryEntry =
            serde_json::from_str(r#"{"pid": 1, "sessionId": "s1", "cwd": "/w"}"#).unwrap();

        // Act
        let observed = ObservedSession::from(entry);

        // Assert
        assert_eq!(observed.session.remote_url, None);
    }
}
