use std::path::PathBuf;

use cerebro_domain::DomainError;
use cerebro_domain::entities::Assignments;
use cerebro_domain::ports::AssignmentStore;
use cerebro_protocol::{read_json, write_json_atomic};

use crate::models::AssignmentsFile;

/// Persists room and character picks so they survive app restarts.
pub struct JsonAssignmentStore {
    file: PathBuf,
}

impl JsonAssignmentStore {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self { file: file.into() }
    }
}

impl AssignmentStore for JsonAssignmentStore {
    fn load(&self) -> Assignments {
        // A missing or corrupted file just means "no memory yet".
        read_json::<AssignmentsFile>(&self.file)
            .map(Assignments::from)
            .unwrap_or_default()
    }

    fn save(&self, assignments: &Assignments) -> Result<(), DomainError> {
        write_json_atomic(&self.file, &AssignmentsFile::from(assignments))
            .map_err(|e| DomainError::Persistence(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerebro_domain::entities::{ActorKey, CharacterId, RoomId, SessionId};

    #[test]
    fn round_trips_and_tolerates_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonAssignmentStore::new(dir.path().join("assignments.json"));
        assert_eq!(store.load(), Assignments::default());

        let mut a = Assignments::default();
        a.rooms.insert(SessionId::new("s"), RoomId::new("garage"));
        a.characters.insert(
            ActorKey::Lead(SessionId::new("s")),
            CharacterId::new("doop"),
        );
        store.save(&a).unwrap();
        assert_eq!(store.load(), a);
    }
}
