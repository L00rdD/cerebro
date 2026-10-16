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
    fn from(a: &Assignments) -> Self {
        Self {
            version: 1,
            rooms: a
                .rooms
                .iter()
                .map(|(s, r)| (s.to_string(), r.to_string()))
                .collect(),
            characters: a
                .characters
                .iter()
                .map(|(k, c)| (k.encode(), c.to_string()))
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
                .map(|(s, r)| (SessionId::new(s), RoomId::new(r)))
                .collect(),
            characters: file
                .characters
                .into_iter()
                .filter_map(|(k, c)| Some((ActorKey::decode(&k)?, CharacterId::new(c))))
                .collect(),
        }
    }
}
