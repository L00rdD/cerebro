use std::net::{IpAddr, Ipv4Addr};

use cerebro_domain::DomainError;
use cerebro_domain::ports::{NetworkStatus, PrivateNetwork};

/// Interfaces that are not the network the Mac is plugged into: VPN tunnels (`ZeroTier` has
/// its own entry), container and virtual machine bridges, Internet Sharing.
const VIRTUAL_PREFIXES: &[&str] = &[
    "feth",
    "zt",
    "utun",
    "tun",
    "tap",
    "wg",
    "tailscale",
    "docker",
    "br-",
    "veth",
    "virbr",
    "vmnet",
    "vboxnet",
    "bridge",
    "awdl",
    "llw",
];
/// Windows names its adapters after their product.
const VIRTUAL_NAMES: &[&str] = &["zerotier", "vethernet", "virtualbox", "vmware", "hyper-v"];

/// The Wi-Fi or Ethernet network the Mac is on, home or office: phones on it reach Cerebro
/// without any VPN. Traffic is not encrypted, so it is off until the user turns it on.
pub struct LocalNetwork;

impl PrivateNetwork for LocalNetwork {
    fn id(&self) -> &'static str {
        "local"
    }

    fn name(&self) -> String {
        "Local network".into()
    }

    /// The page and its key travel in clear text: anyone on a café Wi-Fi could read them.
    fn on_by_default(&self) -> bool {
        false
    }

    fn status(&self) -> Result<NetworkStatus, DomainError> {
        let interfaces = if_addrs::get_if_addrs().map_err(|error| {
            DomainError::Platform(format!("cannot list network interfaces: {error}"))
        })?;
        let addresses = lan_addresses(
            interfaces
                .iter()
                .filter(|interface| interface.is_oper_up() && !interface.is_p2p())
                .map(|interface| (interface.name.as_str(), interface.ip())),
        );
        Ok(if addresses.is_empty() {
            NetworkStatus::Offline
        } else {
            NetworkStatus::Online(addresses)
        })
    }

    /// Always there: the OS manages it.
    fn start(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

/// The private IPv4 addresses of physical interfaces. IPv6 link-local addresses would need a
/// zone in the link, and public addresses are never served.
pub fn lan_addresses<'a>(interfaces: impl IntoIterator<Item = (&'a str, IpAddr)>) -> Vec<IpAddr> {
    interfaces
        .into_iter()
        .filter(|(name, _)| !is_virtual(name))
        .filter_map(|(_, ip)| match ip {
            IpAddr::V4(v4) if is_lan(v4) => Some(ip),
            _ => None,
        })
        .collect()
}

fn is_lan(ip: Ipv4Addr) -> bool {
    ip.is_private() && !ip.is_loopback()
}

fn is_virtual(name: &str) -> bool {
    let name = name.to_lowercase();
    VIRTUAL_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
        || VIRTUAL_NAMES.iter().any(|product| name.contains(product))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    #[test]
    fn keeps_private_addresses_of_physical_interfaces() {
        // Arrange
        let interfaces = [
            ("en0", ip("192.168.1.20")),
            ("en7", ip("10.0.0.5")),
            ("Wi-Fi", ip("172.20.10.2")),
        ];

        // Act
        let addresses = lan_addresses(interfaces);

        // Assert
        assert_eq!(
            addresses,
            [ip("192.168.1.20"), ip("10.0.0.5"), ip("172.20.10.2")]
        );
    }

    #[test]
    fn leaves_out_vpns_and_virtual_bridges() {
        // Arrange
        let interfaces = [
            ("feth1234", ip("10.147.17.42")),
            ("ztabcdef12", ip("10.147.17.43")),
            ("ZeroTier One [0123456789abcdef]", ip("10.147.17.44")),
            ("utun3", ip("10.8.0.2")),
            ("docker0", ip("172.17.0.1")),
            ("bridge100", ip("192.168.2.1")),
            ("vEthernet (WSL)", ip("172.29.0.1")),
        ];

        // Act
        let addresses = lan_addresses(interfaces);

        // Assert
        assert_eq!(addresses, Vec::<IpAddr>::new());
    }

    #[test]
    fn leaves_out_loopback_public_link_local_and_ipv6() {
        // Arrange
        let interfaces = [
            ("lo0", ip("127.0.0.1")),
            ("en0", ip("203.0.113.7")),
            ("en0", ip("169.254.10.1")),
            ("en0", ip("fd00::1")),
            ("en0", ip("fe80::1")),
        ];

        // Act
        let addresses = lan_addresses(interfaces);

        // Assert
        assert_eq!(addresses, Vec::<IpAddr>::new());
    }
}
