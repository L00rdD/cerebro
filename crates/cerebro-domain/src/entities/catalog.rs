use super::{CharacterId, RoomId};
use crate::DomainError;

/// Ids of the characters and rooms available in the assets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    characters: Vec<CharacterId>,
    rooms: Vec<RoomId>,
}

impl Catalog {
    pub fn new(characters: Vec<CharacterId>, rooms: Vec<RoomId>) -> Result<Self, DomainError> {
        if characters.is_empty() || rooms.is_empty() {
            return Err(DomainError::InvalidCatalog(
                "at least one character and one room are required".into(),
            ));
        }
        Ok(Self { characters, rooms })
    }

    pub fn characters(&self) -> &[CharacterId] {
        &self.characters
    }

    pub fn rooms(&self) -> &[RoomId] {
        &self.rooms
    }
}
