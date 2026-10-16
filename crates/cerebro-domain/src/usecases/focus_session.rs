use std::sync::Arc;

use crate::DomainError;
use crate::entities::Session;
use crate::ports::{FocusOutcome, FocusTarget, TerminalFocuser};

/// Brings the terminal hosting a session to the front (click on a card).
pub struct FocusSession {
    focuser: Arc<dyn TerminalFocuser>,
}

impl FocusSession {
    pub fn new(focuser: Arc<dyn TerminalFocuser>) -> Self {
        Self { focuser }
    }

    pub fn execute(&self, session: &Session) -> Result<FocusOutcome, DomainError> {
        let pid = session.pid.ok_or_else(|| {
            DomainError::NotFocusable(session.id.to_string(), "no process id (hook-only session)".into())
        })?;
        self.focuser.focus(&FocusTarget { pid, cwd: session.cwd.clone() })
    }
}
