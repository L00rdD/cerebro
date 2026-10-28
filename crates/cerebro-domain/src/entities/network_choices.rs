use std::collections::BTreeMap;

/// Networks the user turned on or off, by network id. A network missing here keeps its default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkChoices {
    pub on: BTreeMap<String, bool>,
}

impl NetworkChoices {
    pub fn is_on(&self, network: &str, default: bool) -> bool {
        self.on.get(network).copied().unwrap_or(default)
    }

    pub fn set(&mut self, network: &str, on: bool) {
        self.on.insert(network.to_owned(), on);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_choice_overrides_the_default() {
        // Arrange
        let mut choices = NetworkChoices::default();

        // Act
        choices.set("local", true);
        choices.set("zerotier", false);

        // Assert
        assert!(choices.is_on("local", false));
        assert!(!choices.is_on("zerotier", true));
    }

    #[test]
    fn an_untouched_network_keeps_its_default() {
        // Arrange
        let choices = NetworkChoices::default();

        // Act & Assert
        assert!(choices.is_on("zerotier", true));
        assert!(!choices.is_on("local", false));
    }
}
