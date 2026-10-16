use cerebro_domain::DomainError;
use cerebro_domain::entities::{Catalog, CharacterId, RoomId};
use cerebro_domain::ports::CatalogSource;

use crate::models::{CharactersFile, RoomsFile};

/// The asset manifests, compiled into the binary so backend and UI always agree.
const CHARACTERS_JSON: &str = include_str!("../../../../assets/characters/characters.json");
const ROOMS_JSON: &str = include_str!("../../../../assets/rooms/rooms.json");

pub struct EmbeddedCatalog;

impl CatalogSource for EmbeddedCatalog {
    fn catalog(&self) -> Result<Catalog, DomainError> {
        let invalid = |e: serde_json::Error| DomainError::InvalidCatalog(e.to_string());
        let characters: CharactersFile = serde_json::from_str(CHARACTERS_JSON).map_err(invalid)?;
        let rooms: RoomsFile = serde_json::from_str(ROOMS_JSON).map_err(invalid)?;
        Catalog::new(
            characters.characters.into_iter().map(|c| CharacterId::new(c.id)).collect(),
            rooms.rooms.into_iter().map(|r| RoomId::new(r.id)).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_manifests_are_valid() {
        let catalog = EmbeddedCatalog.catalog().unwrap();
        assert!(catalog.characters().len() >= 70);
        assert!(catalog.rooms().len() >= 20);
    }
}
