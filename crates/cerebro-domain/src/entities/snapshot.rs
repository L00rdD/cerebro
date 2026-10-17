use super::{Scene, ToolId};
use crate::time::Timestamp;

/// A session source that failed or returned degraded data. Its other sessions are still shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceWarning {
    pub tool: ToolId,
    pub message: String,
}

/// Everything the UI needs at a given instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub scenes: Vec<Scene>,
    /// Empty when every source answered normally.
    pub warnings: Vec<SourceWarning>,
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
        self.scenes == other.scenes && self.warnings == other.warnings
    }
}
