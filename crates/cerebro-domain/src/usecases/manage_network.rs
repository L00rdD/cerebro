use std::sync::Arc;

use crate::DomainError;
use crate::ports::{NetworkStatus, PrivateNetwork};

/// Reports and starts the private network the phone reaches Cerebro through.
pub struct ManageNetwork {
    network: Arc<dyn PrivateNetwork>,
}

impl ManageNetwork {
    pub fn new(network: Arc<dyn PrivateNetwork>) -> Self {
        Self { network }
    }

    pub fn name(&self) -> String {
        self.network.name()
    }

    pub fn status(&self) -> Result<NetworkStatus, DomainError> {
        self.network.status()
    }

    /// Starts the network when it is installed but stopped. Does nothing otherwise.
    pub fn start(&self) -> Result<NetworkStatus, DomainError> {
        match self.network.status()? {
            NetworkStatus::Stopped => {
                self.network.start()?;
                self.network.status()
            }
            NetworkStatus::NotInstalled => Err(DomainError::Platform(format!(
                "{} is not installed",
                self.network.name()
            ))),
            running => Ok(running),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct FakeNetwork {
        status: Mutex<NetworkStatus>,
        starts: Mutex<usize>,
    }

    impl FakeNetwork {
        fn new(status: NetworkStatus) -> Arc<Self> {
            Arc::new(Self {
                status: Mutex::new(status),
                starts: Mutex::new(0),
            })
        }
    }

    impl PrivateNetwork for FakeNetwork {
        fn name(&self) -> String {
            "FakeNet".into()
        }
        fn status(&self) -> Result<NetworkStatus, DomainError> {
            Ok(self.status.lock().unwrap().clone())
        }
        fn start(&self) -> Result<(), DomainError> {
            *self.starts.lock().unwrap() += 1;
            *self.status.lock().unwrap() = NetworkStatus::Offline;
            Ok(())
        }
    }

    #[test]
    fn starts_a_stopped_network() {
        // Arrange
        let network = FakeNetwork::new(NetworkStatus::Stopped);
        let manage_network = ManageNetwork::new(network.clone());

        // Act
        let status = manage_network.start().unwrap();

        // Assert
        assert_eq!(status, NetworkStatus::Offline);
        assert_eq!(*network.starts.lock().unwrap(), 1);
    }

    #[test]
    fn leaves_a_running_network_alone() {
        // Arrange
        let network = FakeNetwork::new(NetworkStatus::Online(vec![]));
        let manage_network = ManageNetwork::new(network.clone());

        // Act
        let status = manage_network.start().unwrap();

        // Assert
        assert_eq!(status, NetworkStatus::Online(vec![]));
        assert_eq!(*network.starts.lock().unwrap(), 0);
    }

    #[test]
    fn cannot_start_a_missing_network() {
        // Arrange
        let manage_network = ManageNetwork::new(FakeNetwork::new(NetworkStatus::NotInstalled));

        // Act
        let result = manage_network.start();

        // Assert
        assert!(matches!(result, Err(DomainError::Platform(_))));
    }
}
