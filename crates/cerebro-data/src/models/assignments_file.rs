use std::collections::BTreeMap;

use cerebro_domain::entities::{ActorKey, Assignments, CharacterId, RoomId, SessionId};
use serde::{Deserialize, Serialize};

/// `~/.cerebro/assignments.json`.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssignmentsFile {
    pub version: u32,
    pub rooms: BTreeMap<String, String>,
    pub characters: BTreeMap<String, String>,
}

impl From<&Assignments> for AssignmentsFile {
    fn from(assignments: &Assignments) -> Self {
        Self {
            version: 1,
            rooms: assignments
                .rooms
                .iter()
                .map(|(session_id, room_id)| (session_id.to_string(), room_id.to_string()))
                .collect(),
            characters: assignments
                .characters
                .iter()
                .map(|(actor_key, character_id)| (actor_key.encode(), character_id.to_string()))
                .collect(),
        }
    }
}

impl From<AssignmentsFile> for Assignments {
    fn from(file: AssignmentsFile) -> Self {
        Self {
            rooms: file
                .rooms
                .into_iter()
                .map(|(session_id, room_id)| (SessionId::new(session_id), RoomId::new(room_id)))
                .collect(),
            characters: file
                .characters
                .into_iter()
                .filter_map(|(actor_key, character_id)| {
                    Some((
                        ActorKey::decode(&actor_key)?,
                        CharacterId::new(character_id),
                    ))
                })
                .collect(),
        }
    }
}
