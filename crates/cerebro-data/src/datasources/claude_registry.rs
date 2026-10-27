use std::fs;
use std::path::PathBuf;

use cerebro_domain::DomainError;

use super::{ObservedSession, SessionReader};

use crate::models::RegistryEntry;

/// Reads Claude Code's registry: one `<pid>.json` per live interactive session.
pub struct ClaudeRegistrySource {
    dir: PathBuf,
}

impl ClaudeRegistrySource {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// EXERCISE 4: like `observe`, but keeps track of every file that could not be used.
    pub fn scan(&self) -> Result<RegistryScan, DomainError> {
        let entries = fs::read_dir(&self.dir).map_err(|error| {
            DomainError::SourceUnavailable(format!("{}: {error}", self.dir.display()))
        })?;

        Ok(entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .map(|path| {
                fs::read(&path)
                    .map_err(|error| RegistryProblem::Unreadable(error.to_string()))
                    .and_then(|bytes| {
                        serde_json::from_slice::<RegistryEntry>(&bytes)
                            .map_err(|error| RegistryProblem::Malformed(error.to_string()))
                    })
                    .map(ObservedSession::from)
                    .map_err(|error| (path, error))
            })
            .fold(
                RegistryScan {
                    sessions: Vec::new(),
                    problems: Vec::new(),
                },
                |mut scan, result| {
                    match result {
                        Ok(session) => scan.sessions.push(session),
                        Err(problem) => scan.problems.push(problem),
                    }
                    scan
                },
            ))
    }
}

/// Result of a registry scan that does not silently drop bad files (EXERCISE 4).
#[derive(Debug)]
pub struct RegistryScan {
    pub sessions: Vec<ObservedSession>,
    pub problems: Vec<(PathBuf, RegistryProblem)>,
}

/// Why a registry file was skipped (EXERCISE 4: derive `thiserror::Error` for nice messages).
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegistryProblem {
    #[error("unreadable file: {0}")]
    Unreadable(String),
    #[error("malformed file: {0}")]
    Malformed(String),
}

impl SessionReader for ClaudeRegistrySource {
    fn observe(&self) -> Result<Vec<ObservedSession>, DomainError> {
        self.scan()
            .inspect(|scan| {
                for (path, problem) in &scan.problems {
                    eprintln!("{}: {}", path.display(), problem);
                }
            })
            .map(|scan| scan.sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerebro_domain::entities::SessionStatus;

    #[test]
    fn parses_real_registry_files_and_skips_garbage() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("52117.json"),
            include_str!("../../tests/fixtures/registry_52117.json"),
        )
        .unwrap();
        fs::write(dir.path().join("1.json"), "{ not json").unwrap();
        fs::write(dir.path().join("2.key"), "ignored").unwrap();

        let sessions = ClaudeRegistrySource::new(dir.path()).observe().unwrap();
        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.session.pid, Some(52117));
        assert_eq!(session.session.name, "example-project-f6");
        assert_eq!(session.session.status, SessionStatus::Idle);
        assert_eq!(
            session.process_start.as_deref(),
            Some("Thu Oct 15 08:43:57 2026")
        );
    }

    #[test]
    fn scan_reports_malformed_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("52117.json"),
            include_str!("../../tests/fixtures/registry_52117.json"),
        )
        .unwrap();
        fs::write(dir.path().join("1.json"), "{ not json").unwrap();
        fs::write(
            dir.path().join("2.key"),
            "not a registry file: ignored, not a problem",
        )
        .unwrap();

        let scan = ClaudeRegistrySource::new(dir.path()).scan().unwrap();
        assert_eq!(scan.sessions.len(), 1);
        assert_eq!(scan.problems.len(), 1);
        assert!(scan.problems[0].0.ends_with("1.json"));
        assert!(matches!(scan.problems[0].1, RegistryProblem::Malformed(_)));
    }

    #[test]
    fn missing_directory_is_reported() {
        let source = ClaudeRegistrySource::new("/definitely/not/here");
        assert!(matches!(
            source.observe(),
            Err(DomainError::SourceUnavailable(_))
        ));
    }
}
