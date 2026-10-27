use std::net::IpAddr;

use serde::Deserialize;

/// `zerotier-cli -j info`: the node itself.
#[derive(Debug, Clone, Deserialize)]
pub struct ZeroTierInfo {
    #[serde(default)]
    pub online: bool,
}

/// One entry of `zerotier-cli -j listnetworks`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroTierNetworkEntry {
    /// `OK` once joined and authorized.
    #[serde(default)]
    pub status: String,
    /// `PRIVATE` (members approved by an admin) or `PUBLIC` (anyone can join).
    #[serde(rename = "type", default)]
    pub kind: String,
    /// CIDR notation, e.g. `10.147.17.42/16`.
    #[serde(default)]
    pub assigned_addresses: Vec<String>,
}

impl ZeroTierNetworkEntry {
    /// This machine's addresses on the network, if it is joined and private.
    /// A public network is open to strangers: Cerebro is never served there.
    pub fn private_addresses(&self) -> Vec<IpAddr> {
        if self.status != "OK" || self.kind != "PRIVATE" {
            return Vec::new();
        }
        self.assigned_addresses
            .iter()
            .filter_map(|cidr| cidr.split('/').next()?.parse().ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(status: &str, kind: &str, addresses: &[&str]) -> ZeroTierNetworkEntry {
        ZeroTierNetworkEntry {
            status: status.into(),
            kind: kind.into(),
            assigned_addresses: addresses.iter().map(|&address| address.into()).collect(),
        }
    }

    #[test]
    fn keeps_the_addresses_of_joined_private_networks() {
        // Arrange
        let joined = entry(
            "OK",
            "PRIVATE",
            &["10.147.17.42/16", "fd00::1/88", "garbage"],
        );

        // Act
        let addresses = joined.private_addresses();

        // Assert
        assert_eq!(
            addresses,
            [
                "10.147.17.42".parse::<IpAddr>().unwrap(),
                "fd00::1".parse().unwrap()
            ]
        );
    }

    #[test]
    fn ignores_public_and_unauthorized_networks() {
        // Act & Assert
        assert_eq!(
            entry("OK", "PUBLIC", &["192.168.195.10/24"]).private_addresses(),
            Vec::<IpAddr>::new()
        );
        assert_eq!(
            entry("ACCESS_DENIED", "PRIVATE", &["10.0.0.1/24"]).private_addresses(),
            Vec::<IpAddr>::new()
        );
    }
}
