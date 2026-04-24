use log::{debug, info};
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

#[cfg(unix)]
pub fn get_ipv4_system() -> Option<IpAddr> {
    None
}

#[cfg(windows)]
pub fn get_ipv4_system() -> Option<IpAddr> {
    None
}

#[cfg(unix)]
pub fn get_ipv6_system() -> Option<IpAddr> {
    None
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
