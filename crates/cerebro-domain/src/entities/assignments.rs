use std::collections::BTreeMap;

use super::{AgentId, CharacterId, RoomId, SessionId};

/// Identifies a character slot: the lead of a session or one of its subagents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActorKey {
    Lead(SessionId),
    Agent(AgentId),
}

impl ActorKey {
    /// Text form used for hashing and persistence: `lead:<id>` or `agent:<id>`.
    pub fn encode(&self) -> String {
        match self {
            Self::Lead(id) => format!("lead:{id}"),
            Self::Agent(id) => format!("agent:{id}"),
        }
    }

    pub fn decode(raw: &str) -> Option<Self> {
        let (kind, id) = raw.split_once(':')?;
        match kind {
            "lead" => Some(Self::Lead(SessionId::new(id))),
            "agent" => Some(Self::Agent(AgentId::new(id))),
            _ => None,
        }
    }
}

/// Remembered picks, so a session keeps its room and characters for its whole life
/// (and across app restarts once persisted).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Assignments {
    pub rooms: BTreeMap<SessionId, RoomId>,
    pub characters: BTreeMap<ActorKey, CharacterId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_key_round_trips() {
        for key in [ActorKey::Lead(SessionId::new("s")), ActorKey::Agent(AgentId::new("a"))] {
            assert_eq!(ActorKey::decode(&key.encode()), Some(key));
        }
        assert_eq!(ActorKey::decode("nope"), None);
    }
}
