use super::{CharacterId, RoomId, Session};

/// Who plays a session: its room, its lead character and one character per subagent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cast {
    pub room: RoomId,
    pub lead: CharacterId,
    /// Same order as `Session::agents`.
    pub agents: Vec<CharacterId>,
}

/// A session ready to be displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub session: Session,
    pub cast: Cast,
}
