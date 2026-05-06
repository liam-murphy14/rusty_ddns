use log::{debug, info, warn};
use nix::ifaddrs::InterfaceAddress;
use nix::net::if_::InterfaceFlags;
use nix::sys::socket::AddressFamily;
use reqwest::blocking::Client;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub fn get_ipv4() -> Option<IpAddr> {
    #[cfg(unix)]
    if let Some(ip) = get_ipv4_system() {
        return Some(ip);
    }
    #[cfg(windows)]
    if let Some(ip) = get_ipv4_system() {
        return Some(ip);
    }
    #[cfg(not(any(unix, windows)))]
    info!("Operating system family unsupported for system ipv4 fetching, falling back to web");

    info!("Unable to find ipv4 using system fetching, falling back to web");
    get_ipv4_web()
}

pub fn get_ipv6() -> Option<IpAddr> {
    #[cfg(unix)]
    if let Some(ip) = get_ipv6_system() {
        return Some(ip);
    }
    #[cfg(windows)]
    if let Some(ip) = get_ipv6_system() {
        return Some(ip);
    }
    #[cfg(not(any(unix, windows)))]
    info!("Operating system family unsupported for system ipv6 fetching, falling back to web");

    info!("Unable to find ipv6 using system fetching, falling back to web");
    get_ipv6_web()
}

// TODO: find a better way to calculate these
const MAIN_INTERFACE_NAMES: &[&str] = &["eth0", "wlan0", "en0", "eno0", "wlo0"];

fn is_eligible_ipv4(ip: Ipv4Addr) -> bool {
    !ip.is_loopback() && !ip.is_private() && !ip.is_link_local() && !ip.is_unspecified()
}

fn is_eligible_ipv6(ip: Ipv6Addr) -> bool {
    !ip.is_loopback()
        && !ip.is_unique_local()
        && !ip.is_unicast_link_local()
        && !ip.is_unspecified()
}

fn parse_ipv4_web_response(text_ip_address: &str) -> Option<IpAddr> {
    Some(IpAddr::V4(Ipv4Addr::from_str(text_ip_address).ok()?))
}

fn parse_ipv6_web_response(text_ip_address: &str) -> Option<IpAddr> {
    Some(IpAddr::V6(Ipv6Addr::from_str(text_ip_address).ok()?))
}

#[cfg(unix)]
fn get_eligible_addresses(address_family: AddressFamily) -> Vec<InterfaceAddress> {
    use nix::sys::socket::{SockaddrLike, SockaddrStorage};
    match nix::ifaddrs::getifaddrs() {
        Ok(ifaddrs) => ifaddrs
            .filter(|addr| {
                let family = addr.address.as_ref().and_then(SockaddrStorage::family);

                MAIN_INTERFACE_NAMES.contains(&addr.interface_name.as_str())
                    && addr.flags.contains(InterfaceFlags::IFF_RUNNING)
                    && addr.address.is_some()
                    && family.is_some()
                    && family.unwrap().eq(&address_family)
            })
            .collect::<Vec<InterfaceAddress>>(),
        Err(e) => {
            warn!("Error while fetching network interfaces: [{:#?}]", e);
            vec![]
        }
    }
}

#[cfg(unix)]
pub fn get_ipv4_system() -> Option<IpAddr> {
    let eligible = get_eligible_addresses(AddressFamily::Inet);
    let addresses = eligible
        .iter()
        .filter_map(|addr| {
            addr.address
                .and_then(|a| Some(a.as_sockaddr_in()?.ip()))
                .map(|ip| (addr, ip))
        })
        .filter_map(|(addr, ip)| {
            debug!(
                "Found potential ipv4 address [{:?}] on interface [{}] with flags [{:x}<{}>]",
                ip,
                addr.interface_name,
                addr.flags.bits(),
                addr.flags
            );
            match is_eligible_ipv4(ip) {
                true => Some(ip),
                false => None,
            }
        })
        .collect::<Vec<Ipv4Addr>>();

    if !addresses.is_empty() {
        info!(
            "Found ipv4 eligible addresses from system: [{:#?}], using address [{:#?}]",
            addresses,
            addresses.first()
        );
    }

    addresses.first().copied().map(IpAddr::V4)
}

// TODO: windows implementation
#[cfg(windows)]
pub fn get_ipv4_system() -> Option<IpAddr> {
    None
}

#[cfg(unix)]
pub fn get_ipv6_system() -> Option<IpAddr> {
    let eligible = get_eligible_addresses(AddressFamily::Inet6);
    let addresses = eligible
        .iter()
        .filter_map(|addr| {
            addr.address
                .and_then(|a| Some(a.as_sockaddr_in6()?.ip()))
                .map(|ip| (addr, ip))
        })
        .filter_map(|(addr, ip)| {
            debug!(
                "Found potential ipv6 address [{:?}] on interface [{}] with flags [{:x}<{}>]",
                ip,
                addr.interface_name,
                addr.flags.bits(),
                addr.flags
            );
            match is_eligible_ipv6(ip) {
                true => Some(ip),
                false => None,
            }
        })
        .collect::<Vec<Ipv6Addr>>();

    if !addresses.is_empty() {
        // TODO: ideally would like to get the SLAAC address, specifically, or if not present, then the DHCPv6 address, but seems nix crate does not yet support these flags
        info!(
            "Found ipv6 eligible addresses from system: [{:#?}], using address [{:#?}]",
            addresses,
            addresses.first()
        );
    }

    addresses.first().copied().map(IpAddr::V6)
}

// TODO: windows implementation
#[cfg(windows)]
pub fn get_ipv6_system() -> Option<IpAddr> {
    None
}

pub fn get_ipv6_web() -> Option<IpAddr> {
    let client = Client::builder()
        .local_address(Some(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))))
        .build()
        .ok()?;
    let text_ip_address = client
        .get("https://ifconfig.me/ip")
        .send()
        .ok()?
        .text()
        .ok()?;
    debug!("IPv6 address from web is {}", text_ip_address);
    parse_ipv6_web_response(&text_ip_address)
}

pub fn get_ipv4_web() -> Option<IpAddr> {
    let client = Client::builder()
        .local_address(Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))))
        .build()
        .ok()?;
    let text_ip_address = client
        .get("https://ifconfig.me/ip")
        .send()
        .ok()?
        .text()
        .ok()?;
    debug!("IPv4 address from web is {}", text_ip_address);
    parse_ipv4_web_response(&text_ip_address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_ipv4_web_response() {
        assert_eq!(
            parse_ipv4_web_response("203.0.113.10"),
            Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 10)))
        );
    }

    #[test]
    fn rejects_invalid_ipv4_web_response() {
        assert_eq!(parse_ipv4_web_response("not an ip"), None);
        assert_eq!(parse_ipv4_web_response("2001:db8::1"), None);
    }

    #[test]
    fn parses_valid_ipv6_web_response() {
        assert_eq!(
            parse_ipv6_web_response("2001:db8::1"),
            Some(IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1)))
        );
    }

    #[test]
    fn rejects_invalid_ipv6_web_response() {
        assert_eq!(parse_ipv6_web_response("not an ip"), None);
        assert_eq!(parse_ipv6_web_response("203.0.113.10"), None);
    }

    #[test]
    fn identifies_eligible_ipv4_addresses() {
        assert!(is_eligible_ipv4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_eligible_ipv4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(!is_eligible_ipv4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(!is_eligible_ipv4(Ipv4Addr::new(172, 16, 0, 1)));
        assert!(!is_eligible_ipv4(Ipv4Addr::new(192, 168, 0, 1)));
        assert!(!is_eligible_ipv4(Ipv4Addr::new(169, 254, 1, 1)));
        assert!(!is_eligible_ipv4(Ipv4Addr::new(0, 0, 0, 0)));
    }

    #[test]
    fn identifies_eligible_ipv6_addresses() {
        assert!(is_eligible_ipv6(Ipv6Addr::new(
            0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888
        )));
        assert!(!is_eligible_ipv6(Ipv6Addr::LOCALHOST));
        assert!(!is_eligible_ipv6(Ipv6Addr::new(
            0xfc00, 0, 0, 0, 0, 0, 0, 1
        )));
        assert!(!is_eligible_ipv6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        )));
        assert!(!is_eligible_ipv6(Ipv6Addr::UNSPECIFIED));
    }
}
