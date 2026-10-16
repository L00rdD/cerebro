use serde::Deserialize;

/// `assets/characters/characters.json` (only the fields the backend needs).
#[derive(Debug, Deserialize)]
pub struct CharactersFile {
    pub characters: Vec<IdEntry>,
}

/// `assets/rooms/rooms.json` (only the fields the backend needs).
#[derive(Debug, Deserialize)]
pub struct RoomsFile {
    pub rooms: Vec<IdEntry>,
}

#[derive(Debug, Deserialize)]
pub struct IdEntry {
    pub id: String,
}
