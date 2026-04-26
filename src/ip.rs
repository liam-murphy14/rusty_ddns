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
        .filter_map(|addr| addr.address.and_then(|a| Some(a.as_sockaddr_in()?.ip())))
        .filter(|ip| {
            debug!("Found potential ipv4 address [{:?}]", ip);
            !ip.is_loopback() && !ip.is_private() && !ip.is_link_local() && !ip.is_unspecified()
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

#[cfg(windows)]
pub fn get_ipv4_system() -> Option<IpAddr> {
    None
}

#[cfg(unix)]
pub fn get_ipv6_system() -> Option<IpAddr> {
    let eligible = get_eligible_addresses(AddressFamily::Inet6);
    let addresses = eligible
        .iter()
        .filter_map(|addr| addr.address.and_then(|a| Some(a.as_sockaddr_in6()?.ip())))
        .filter(|ip| {
            debug!("Found potential ipv6 address [{:?}]", ip);
            !ip.is_loopback()
                && !ip.is_unique_local()
                && !ip.is_unicast_link_local()
                && !ip.is_unspecified()
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
    Some(IpAddr::V6(Ipv6Addr::from_str(&text_ip_address).ok()?))
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
    Some(IpAddr::V4(Ipv4Addr::from_str(&text_ip_address).ok()?))
}
