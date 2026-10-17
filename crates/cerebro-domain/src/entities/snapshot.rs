use super::Scene;
use crate::time::Timestamp;

/// Which source the sessions were taken from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataMode {
    /// Normal mode: Claude Code's registry, enriched by hooks.
    Registry,
    /// The registry is unreadable or looks broken; sessions come from hooks only.
    HooksFallback { reason: String },
}

/// Everything the UI needs at a given instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub scenes: Vec<Scene>,
    pub mode: DataMode,
    pub taken_at: Timestamp,
}

impl Snapshot {
    pub fn character_count(&self) -> usize {
        self.scenes
            .iter()
            .map(|scene| scene.session.character_count())
            .sum()
    }

    pub fn agent_count(&self) -> usize {
        self.scenes
            .iter()
            .map(|scene| scene.session.agents.len())
            .sum()
    }

    pub fn busy_count(&self) -> usize {
        self.scenes
            .iter()
            .filter(|scene| scene.session.is_busy())
            .count()
    }

    /// Same content regardless of when it was taken; used to skip redundant UI updates.
    pub fn same_content(&self, other: &Snapshot) -> bool {
        self.scenes == other.scenes && self.mode == other.mode
    }
}
