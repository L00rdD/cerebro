use std::path::PathBuf;

use cerebro_domain::DomainError;
use cerebro_domain::entities::NetworkChoices;
use cerebro_domain::ports::NetworkChoiceStore;
use cerebro_protocol::{read_json, write_json_atomic};

use crate::models::PhoneNetworksFile;

/// Persists which networks the phone page is served on.
pub struct JsonNetworkChoiceStore {
    file: PathBuf,
}

impl JsonNetworkChoiceStore {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self { file: file.into() }
    }
}

impl NetworkChoiceStore for JsonNetworkChoiceStore {
    fn load(&self) -> NetworkChoices {
        // A missing or corrupted file just means "every network at its default".
        read_json::<PhoneNetworksFile>(&self.file)
            .map(NetworkChoices::from)
            .unwrap_or_default()
    }

    fn save(&self, choices: &NetworkChoices) -> Result<(), DomainError> {
        write_json_atomic(&self.file, &PhoneNetworksFile::from(choices))
            .map_err(|error| DomainError::Persistence(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_tolerates_missing_file() {
        // Arrange
        let dir = tempfile::tempdir().unwrap();
        let store = JsonNetworkChoiceStore::new(dir.path().join("phone-networks.json"));

        // Act & Assert
        assert_eq!(store.load(), NetworkChoices::default());

        // Arrange
        let mut choices = NetworkChoices::default();
        choices.set("local", true);
        choices.set("zerotier", false);

        // Act
        store.save(&choices).unwrap();

        // Assert
        assert_eq!(store.load(), choices);
    }
}
