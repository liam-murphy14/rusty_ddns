use crate::cloudflare::{CloudflareUpdateRequest, handle_update};
use reqwest::Error;
use std::net::IpAddr;

pub enum UpdateRequest {
    Cloudflare(CloudflareUpdateRequest),
}
impl UpdateRequest {
    pub fn cloudflare(
        api_token: String,
        record_name: String,
        ipv4addr: Option<IpAddr>,
        ipv6addr: Option<IpAddr>,
        allow_create: bool,
    ) -> Self {
        UpdateRequest::Cloudflare(CloudflareUpdateRequest::new(
            api_token,
            record_name,
            ipv4addr,
            ipv6addr,
            allow_create,
        ))
    }
}
#[derive(Debug)]
pub struct UpdateResponse {
    pub ipv4_update: Option<RecordUpdate>,
    pub ipv6_update: Option<RecordUpdate>,
}
#[derive(Debug)]
pub struct RecordUpdate {
    pub ip: IpAddr,
    pub record_name: String,
    pub modified_on: String,
}
#[derive(Debug)]
pub enum UpdateError {
    Retryable(String),
    Fatal(String),
}

pub fn translate_error(e: Error) -> UpdateError {
    if e.is_timeout() {
        return UpdateError::Retryable(format!("Connection timed out, error: [{:#?}]", e));
    }
    if e.is_connect() {
        return UpdateError::Retryable(format!("Connection error, error: [{:#?}]", e));
    }
    UpdateError::Fatal(format!("Got unexpected error, error: [{:#?}]", e))
}

pub fn update_record(request: UpdateRequest) -> Result<UpdateResponse, UpdateError> {
    match request {
        UpdateRequest::Cloudflare(request) => handle_update(request),
    }
}

// record name already validated
pub fn get_sld(valid_record_name: &str) -> String {
    let names = valid_record_name.split('.').collect::<Vec<_>>();
    names[names.len() - 2..].join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn get_sld_returns_domain_for_root_record() {
        assert_eq!(get_sld("example.com"), "example.com");
    }

    #[test]
    fn get_sld_strips_subdomains() {
        assert_eq!(get_sld("host.internal.example.com"), "example.com");
    }

    #[test]
    fn cloudflare_constructor_wraps_cloudflare_request() {
        let ipv4addr = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 10));
        let ipv6addr = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));

        let request = UpdateRequest::cloudflare(
            "token".to_string(),
            "host.example.com".to_string(),
            Some(ipv4addr),
            Some(ipv6addr),
            true,
        );

        assert!(matches!(request, UpdateRequest::Cloudflare(_)));
    }
}
