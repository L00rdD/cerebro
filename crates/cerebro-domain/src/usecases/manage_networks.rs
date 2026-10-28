use std::sync::Arc;

use crate::DomainError;
use crate::ports::{NetworkChoiceStore, NetworkStatus, PrivateNetwork};

/// Reports, turns on or off, and starts the networks phones reach Cerebro through.
pub struct ManageNetworks {
    networks: Vec<Arc<dyn PrivateNetwork>>,
    choices: Arc<dyn NetworkChoiceStore>,
}

/// One network as the user sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkCheck {
    pub id: &'static str,
    pub name: String,
    /// `None` when the user turned the network off: it isn't even queried.
    pub status: Option<Result<NetworkStatus, DomainError>>,
}

impl ManageNetworks {
    pub fn new(
        networks: Vec<Arc<dyn PrivateNetwork>>,
        choices: Arc<dyn NetworkChoiceStore>,
    ) -> Self {
        Self { networks, choices }
    }

    /// Every network, in the order given at construction. Queries only the ones turned on.
    pub fn check(&self) -> Vec<NetworkCheck> {
        let choices = self.choices.load();
        self.networks
            .iter()
            .map(|network| NetworkCheck {
                id: network.id(),
                name: network.name(),
                status: choices
                    .is_on(network.id(), network.on_by_default())
                    .then(|| network.status()),
            })
            .collect()
    }

    pub fn set_on(&self, id: &str, on: bool) -> Result<(), DomainError> {
        let network = self.find(id)?;
        let mut choices = self.choices.load();
        choices.set(network.id(), on);
        self.choices.save(&choices)
    }

    /// Starts the network when it is installed but stopped. Does nothing otherwise.
    pub fn start(&self, id: &str) -> Result<NetworkStatus, DomainError> {
        let network = self.find(id)?;
        match network.status()? {
            NetworkStatus::Stopped => {
                network.start()?;
                network.status()
            }
            NetworkStatus::NotInstalled => Err(DomainError::Platform(format!(
                "{} is not installed",
                network.name()
            ))),
            running => Ok(running),
        }
    }

    fn find(&self, id: &str) -> Result<&Arc<dyn PrivateNetwork>, DomainError> {
        self.networks
            .iter()
            .find(|network| network.id() == id)
            .ok_or_else(|| DomainError::UnknownNetwork(id.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::entities::NetworkChoices;

    struct FakeNetwork {
        id: &'static str,
        on_by_default: bool,
        status: Mutex<NetworkStatus>,
        queries: Mutex<usize>,
        starts: Mutex<usize>,
    }

    impl FakeNetwork {
        fn new(id: &'static str, on_by_default: bool, status: NetworkStatus) -> Arc<Self> {
            Arc::new(Self {
                id,
                on_by_default,
                status: Mutex::new(status),
                queries: Mutex::new(0),
                starts: Mutex::new(0),
            })
        }
    }

    impl PrivateNetwork for FakeNetwork {
        fn id(&self) -> &'static str {
            self.id
        }
        fn name(&self) -> String {
            format!("Fake {}", self.id)
        }
        fn on_by_default(&self) -> bool {
            self.on_by_default
        }
        fn status(&self) -> Result<NetworkStatus, DomainError> {
            *self.queries.lock().unwrap() += 1;
            Ok(self.status.lock().unwrap().clone())
        }
        fn start(&self) -> Result<(), DomainError> {
            *self.starts.lock().unwrap() += 1;
            *self.status.lock().unwrap() = NetworkStatus::Offline;
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemoryChoices(Mutex<NetworkChoices>);

    impl NetworkChoiceStore for MemoryChoices {
        fn load(&self) -> NetworkChoices {
            self.0.lock().unwrap().clone()
        }
        fn save(&self, choices: &NetworkChoices) -> Result<(), DomainError> {
            *self.0.lock().unwrap() = choices.clone();
            Ok(())
        }
    }

    fn manage(networks: Vec<Arc<FakeNetwork>>) -> ManageNetworks {
        ManageNetworks::new(
            networks
                .into_iter()
                .map(|network| network as Arc<dyn PrivateNetwork>)
                .collect(),
            Arc::new(MemoryChoices::default()),
        )
    }

    #[test]
    fn queries_only_the_networks_turned_on() {
        // Arrange
        let vpn = FakeNetwork::new("vpn", true, NetworkStatus::Online(vec![]));
        let local = FakeNetwork::new("local", false, NetworkStatus::Online(vec![]));
        let manage_networks = manage(vec![vpn.clone(), local.clone()]);

        // Act
        let checks = manage_networks.check();

        // Assert
        assert_eq!(
            checks,
            [
                NetworkCheck {
                    id: "vpn",
                    name: "Fake vpn".into(),
                    status: Some(Ok(NetworkStatus::Online(vec![]))),
                },
                NetworkCheck {
                    id: "local",
                    name: "Fake local".into(),
                    status: None,
                },
            ]
        );
        assert_eq!(*local.queries.lock().unwrap(), 0);
    }

    #[test]
    fn remembers_the_networks_turned_on_and_off() {
        // Arrange
        let vpn = FakeNetwork::new("vpn", true, NetworkStatus::Offline);
        let local = FakeNetwork::new("local", false, NetworkStatus::Offline);
        let manage_networks = manage(vec![vpn, local]);

        // Act
        manage_networks.set_on("vpn", false).unwrap();
        manage_networks.set_on("local", true).unwrap();

        // Assert
        let on = manage_networks
            .check()
            .iter()
            .map(|check| check.status.is_some())
            .collect::<Vec<_>>();
        assert_eq!(on, [false, true]);
    }

    #[test]
    fn cannot_turn_on_an_unknown_network() {
        // Arrange
        let manage_networks = manage(vec![]);

        // Act
        let result = manage_networks.set_on("nope", true);

        // Assert
        assert_eq!(result, Err(DomainError::UnknownNetwork("nope".into())));
    }

    #[test]
    fn starts_a_stopped_network() {
        // Arrange
        let network = FakeNetwork::new("vpn", true, NetworkStatus::Stopped);
        let manage_networks = manage(vec![network.clone()]);

        // Act
        let status = manage_networks.start("vpn").unwrap();

        // Assert
        assert_eq!(status, NetworkStatus::Offline);
        assert_eq!(*network.starts.lock().unwrap(), 1);
    }

    #[test]
    fn leaves_a_running_network_alone() {
        // Arrange
        let network = FakeNetwork::new("vpn", true, NetworkStatus::Online(vec![]));
        let manage_networks = manage(vec![network.clone()]);

        // Act
        let status = manage_networks.start("vpn").unwrap();

        // Assert
        assert_eq!(status, NetworkStatus::Online(vec![]));
        assert_eq!(*network.starts.lock().unwrap(), 0);
    }

    #[test]
    fn cannot_start_a_missing_network() {
        // Arrange
        let manage_networks = manage(vec![FakeNetwork::new(
            "vpn",
            true,
            NetworkStatus::NotInstalled,
        )]);

        // Act
        let result = manage_networks.start("vpn");

        // Assert
        assert!(matches!(result, Err(DomainError::Platform(_))));
    }
}
