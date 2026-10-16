//! Contracts the domain needs from the outside world (the "repositories" of the template).
//!
//! The data layer implements them; tests implement them with in-memory fakes.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::DomainError;
use crate::entities::{Agent, Assignments, Catalog, Session, SessionId};
use crate::time::Timestamp;

/// A session as read from a source, before process verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedSession {
    pub session: Session,
    /// Process start time as recorded by the source (e.g. `"Thu Oct 15 08:43:57 2026"`),
    /// used to detect a recycled pid.
    pub process_start: Option<String>,
}

/// Lists live sessions (Claude Code registry, Cerebro hook records…).
pub trait SessionSource: Send + Sync {
    fn observe(&self) -> Result<Vec<ObservedSession>, DomainError>;
}

/// Running subagents, recorded by the hook.
pub trait AgentSource: Send + Sync {
    fn agents_of(&self, session: &SessionId) -> Vec<Agent>;
    /// Drops leftovers of sessions that are gone without a `SessionEnd` event.
    fn forget_all_except(&self, alive: &[SessionId]);
}

/// What the OS says about a running process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    /// Same format as `ObservedSession::process_start`; `None` if the platform can't tell.
    pub start: Option<String>,
}

pub trait ProcessInspector: Send + Sync {
    /// Returns an entry for each pid that is currently alive.
    fn inspect(&self, pids: &[u32]) -> HashMap<u32, ProcessInfo>;
}

pub trait AssignmentStore: Send + Sync {
    fn load(&self) -> Assignments;
    fn save(&self, assignments: &Assignments) -> Result<(), DomainError>;
}

pub trait CatalogSource: Send + Sync {
    fn catalog(&self) -> Result<Catalog, DomainError>;
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// Brings the terminal (or editor) hosting a session to the front.
pub trait TerminalFocuser: Send + Sync {
    fn focus(&self, target: &FocusTarget) -> Result<FocusOutcome, DomainError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusTarget {
    pub pid: u32,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusOutcome {
    /// The exact tab or pane was selected.
    Tab { app: String },
    /// Only the hosting application could be activated.
    App { app: String },
    /// No known host was found (tmux, ssh, unsupported terminal…).
    NotFound,
}

/// State of Cerebro's entries in Claude Code's settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookSetup {
    Installed,
    Missing,
    /// Installed, but pointing to an old binary or missing some events.
    Outdated,
}

pub trait HookInstaller: Send + Sync {
    fn status(&self) -> Result<HookSetup, DomainError>;
    fn install(&self) -> Result<(), DomainError>;
}
