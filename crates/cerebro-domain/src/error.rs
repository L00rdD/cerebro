use thiserror::Error;

/// Every failure the domain can report. Adapters convert their own errors into these.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// A data source could not be read at all (missing directory, permission…).
    #[error("data source unavailable: {0}")]
    SourceUnavailable(String),

    #[error("could not persist data: {0}")]
    Persistence(String),

    #[error("asset catalog is invalid: {0}")]
    InvalidCatalog(String),

    #[error("session {0} cannot be focused: {1}")]
    NotFocusable(String, String),

    /// An OS integration failed (osascript, ps, settings file…).
    #[error("platform error: {0}")]
    Platform(String),

    #[error("unknown session {0}")]
    UnknownSession(String),

    #[error("unknown network {0}")]
    UnknownNetwork(String),
}
