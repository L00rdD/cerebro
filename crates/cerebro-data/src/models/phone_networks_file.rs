use std::collections::BTreeMap;

use cerebro_domain::entities::NetworkChoices;
use serde::{Deserialize, Serialize};

/// `~/.cerebro/phone-networks.json`: `{ "version": 1, "on": { "local": true } }`.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PhoneNetworksFile {
    pub version: u32,
    #[serde(default)]
    pub on: BTreeMap<String, bool>,
}

impl From<&NetworkChoices> for PhoneNetworksFile {
    fn from(choices: &NetworkChoices) -> Self {
        Self {
            version: 1,
            on: choices.on.clone(),
        }
    }
}

impl From<PhoneNetworksFile> for NetworkChoices {
    fn from(file: PhoneNetworksFile) -> Self {
        Self { on: file.on }
    }
}
