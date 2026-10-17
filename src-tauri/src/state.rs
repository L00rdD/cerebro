use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use cerebro_domain::entities::{Scene, Snapshot};

use crate::injector::Container;

pub struct AppState {
    pub container: Container,
    last_snapshot: Mutex<Option<Snapshot>>,
    /// When the panel was last hidden; lets a tray click that caused the blur not reopen it.
    pub last_hidden: Mutex<Option<Instant>>,
    /// Drives the refresh rate and pauses UI animations while hidden.
    pub panel_visible: AtomicBool,
}

impl AppState {
    pub fn new(container: Container) -> Self {
        Self {
            container,
            last_snapshot: Mutex::new(None),
            last_hidden: Mutex::new(None),
            panel_visible: AtomicBool::new(false),
        }
    }

    /// Stores `snapshot`; returns `true` when its content differs from the previous one.
    pub fn replace_snapshot(&self, snapshot: Snapshot) -> bool {
        let mut last = self
            .last_snapshot
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let changed = last
            .as_ref()
            .is_none_or(|previous| !previous.same_content(&snapshot));
        *last = Some(snapshot);
        changed
    }

    pub fn snapshot(&self) -> Option<Snapshot> {
        self.last_snapshot
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn scene(&self, session_id: &str) -> Option<Scene> {
        self.snapshot()?
            .scenes
            .into_iter()
            .find(|scene| scene.session.id.as_str() == session_id)
    }
}
