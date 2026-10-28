//! The contract between `cerebro-hook` (writer) and the Cerebro app (reader).
//!
//! Both sides depend on this crate so the on-disk format can only change in one place.
//!
//! Layout under `~/.cerebro`:
//! ```text
//! sessions/<session_id>.json            one HookSession per Claude Code session
//! agents/<session_id>/<agent_id>.json   one HookAgent per running subagent
//! assignments.json                      stable room / character picks (app only)
//! mobile-key                            secret of the phone link (app only)
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Bump when the record layout changes in a non backward compatible way.
pub const FORMAT_VERSION: u32 = 1;

/// Claude Code hook events the hook handles. The installer registers exactly this list.
pub const HOOK_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PostToolUse",
    "Notification",
    "Stop",
    "SessionEnd",
    "SubagentStart",
    "SubagentStop",
];

/// Any hook command containing this marker belongs to Cerebro (current or legacy versions).
pub const HOOK_MARKER: &str = "cerebro-hook";

/// Status of a session as seen by the hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookStatus {
    Busy,
    Idle,
    /// Claude is blocked on a permission prompt.
    Waiting,
}

/// What the hook knows about a session. Written to `sessions/<session_id>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookSession {
    pub version: u32,
    pub session_id: String,
    pub cwd: String,
    pub status: HookStatus,
    /// Unix time in milliseconds.
    pub started_at_ms: u64,
    /// Unix time in milliseconds of the last status change.
    pub status_since_ms: u64,
}

/// A running subagent. Written to `agents/<session_id>/<agent_id>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookAgent {
    pub version: u32,
    pub agent_id: String,
    pub agent_type: String,
    pub started_at_ms: u64,
}

/// Every path Cerebro reads or writes, derived from a home directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub home: PathBuf,
}

impl Paths {
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    /// Uses `$HOME` (Unix) or `%USERPROFILE%` (Windows).
    pub fn detect() -> Option<Self> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(Self::new)
    }

    pub fn cerebro_dir(&self) -> PathBuf {
        self.home.join(".cerebro")
    }

    pub fn hook_sessions_dir(&self) -> PathBuf {
        self.cerebro_dir().join("sessions")
    }

    pub fn hook_session_file(&self, session_id: &str) -> PathBuf {
        self.hook_sessions_dir().join(format!("{session_id}.json"))
    }

    pub fn agents_dir(&self) -> PathBuf {
        self.cerebro_dir().join("agents")
    }

    pub fn session_agents_dir(&self, session_id: &str) -> PathBuf {
        self.agents_dir().join(session_id)
    }

    pub fn agent_file(&self, session_id: &str, agent_id: &str) -> PathBuf {
        self.session_agents_dir(session_id)
            .join(format!("{agent_id}.json"))
    }

    pub fn assignments_file(&self) -> PathBuf {
        self.cerebro_dir().join("assignments.json")
    }

    /// Secret of the phone link (app only). Delete it to revoke every link given so far.
    pub fn mobile_key_file(&self) -> PathBuf {
        self.cerebro_dir().join("mobile-key")
    }

    /// Networks the phone page is served on, as turned on or off by the user (app only).
    pub fn phone_networks_file(&self) -> PathBuf {
        self.cerebro_dir().join("phone-networks.json")
    }

    /// Claude Code's own registry of live sessions (internal, undocumented format).
    pub fn claude_sessions_dir(&self) -> PathBuf {
        self.home.join(".claude").join("sessions")
    }

    pub fn claude_settings_file(&self) -> PathBuf {
        self.home.join(".claude").join("settings.json")
    }
}

/// Keeps only characters that are safe in a file name.
/// Ids come from hook payloads, so they must never be able to escape the directory.
pub fn sanitize_id(raw: &str) -> Option<String> {
    let clean: String = raw
        .chars()
        .filter(|symbol| symbol.is_ascii_alphanumeric() || *symbol == '-' || *symbol == '_')
        .collect();
    (!clean.is_empty()).then_some(clean)
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Writes through a temporary file then renames it, so readers never see half a file.
pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, serde_json::to_vec(value)?)?;
    fs::rename(&temporary_path, path)
}

pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> io::Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_id_strips_path_tricks() {
        // Act & Assert
        assert_eq!(
            sanitize_id("../../etc/passwd").as_deref(),
            Some("etcpasswd")
        );
        assert_eq!(sanitize_id("abc-123_DEF").as_deref(), Some("abc-123_DEF"));
        assert_eq!(sanitize_id("../"), None);
    }

    #[test]
    fn atomic_write_round_trips() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/agent.json");
        let agent = HookAgent {
            version: FORMAT_VERSION,
            agent_id: "a1".into(),
            agent_type: "Explore".into(),
            started_at_ms: 42,
        };

        // Act
        write_json_atomic(&path, &agent).unwrap();

        // Assert
        assert_eq!(read_json::<HookAgent>(&path).unwrap(), agent);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn status_uses_snake_case_on_disk() {
        // Act
        let serialized = serde_json::to_string(&HookStatus::Waiting).unwrap();

        // Assert
        assert_eq!(serialized, "\"waiting\"");
    }
}
