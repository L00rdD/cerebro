//! Contracts the domain needs from the outside world (the "repositories" of the template).
//!
//! The data layer implements them; tests implement them with in-memory fakes.

use std::path::PathBuf;

use crate::DomainError;
use crate::entities::{Assignments, Catalog, Session, ToolId};
use crate::time::Timestamp;

/// Live sessions of one AI tool. The data layer does whatever that tool needs (merging several
/// files, checking processes, attaching subagents) and hands over sessions ready to display.
pub trait SessionSource: Send + Sync {
    /// The tool these sessions belong to, also used to attribute warnings.
    fn tool(&self) -> ToolId;
    fn observe(&self) -> Result<Observation, DomainError>;
}

/// What a [`SessionSource`] saw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observation {
    pub sessions: Vec<Session>,
    /// Set when the source answered with degraded data (e.g. a fallback), shown to the user.
    pub warning: Option<String>,
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

/// State of Cerebro's hook entries in the tool's settings.
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
