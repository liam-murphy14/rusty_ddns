use crate::update::{RecordUpdate, UpdateError, UpdateResponse, get_sld, translate_error};
use log::{debug, info, warn};
use reqwest::{IntoUrl, Method, blocking::Client};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::net::IpAddr;
use std::time::SystemTime;

pub struct CloudflareUpdateRequest {
    api_token: String,
    record_name: String,
    ipv4addr: Option<IpAddr>,
    ipv6addr: Option<IpAddr>,
    allow_create: bool,
}
impl CloudflareUpdateRequest {
    pub fn new(
        api_token: String,
        record_name: String,
        ipv4addr: Option<IpAddr>,
        ipv6addr: Option<IpAddr>,
        allow_create: bool,
    ) -> Self {
        CloudflareUpdateRequest {
            api_token,
            record_name,
            ipv4addr,
            ipv6addr,
            allow_create,
        }
    }
}
#[derive(Deserialize, Debug)]
struct RecordResponse {
    id: String,
    modified_on: String,
}
#[derive(Deserialize, Debug, Clone)]
struct Zone {
    id: String,
    name: String,
}

#[derive(Serialize, Debug)]
struct UpdateRequest {
    name: String,
    content: String,
    comment: String,
    r#type: String,
}

#[derive(Serialize, Debug)]
struct CreateRequest {
    name: String,
    content: String,
    comment: String,
    r#type: String,
    proxied: bool,
}

fn make_request<
    T: DeserializeOwned + Debug,
    U: Serialize + ?Sized + Debug,
    V: Serialize + ?Sized + Debug,
    W: IntoUrl + Debug,
>(
    client: &Client,
    method: Method,
    url: W,
    api_token: &str,
    query: Option<&U>,
    body: Option<&V>,
) -> Result<Option<T>, UpdateError> {
    debug!(
        "Making [{:#?}] request to url: [{:#?}], query: [{:#?}], body: [{:#?}]",
        method, url, query, body
    );
    let mut request_builder = client
        .request(method, url)
        .header("Authorization", format!("Bearer {}", api_token));
    if let Some(query) = query {
        request_builder = request_builder.query(query);
    }
    if let Some(body) = body {
        request_builder = request_builder.json(body);
    }
    let response: Value = match request_builder.send() {
        Ok(response) => match response.json() {
            Ok(response) => response,
            Err(e) => {
                return Err(UpdateError::Fatal(format!(
                    "Failed to parse response, error: [{:#?}]",
                    e
                )));
            }
        },
        Err(e) => {
            return Err(translate_error(e));
        }
    };
    debug!("Response: [{:#?}]", response);
    if let Some(success) = response.get("success")
        && !(success.as_bool().unwrap_or(true))
    {
        return Err(UpdateError::Fatal(format!(
            "Received unsuccessful response from CloudFlare API, response: [{:#?}]",
            response
        )));
    }
    if let Some(errors) = response.get("errors")
        && !errors.as_array().unwrap().is_empty()
    {
        warn!(
            "Received errors from CloudFlare API, errors: [{:#?}]",
            errors
        )
    }
    if let Some(messages) = response.get("messages")
        && !messages.as_array().unwrap().is_empty()
    {
        info!(
            "Received messages from CloudFlare API, errors: [{:#?}]",
            messages
        )
    }
    match response.get("result") {
        Some(result) => match serde_json::from_value(result.clone()) {
            Ok(result) => Ok(Some(result)),
            Err(e) => Err(UpdateError::Fatal(format!(
                "Failed to deserialize response: [{:#?}], error: [{:#?}]",
                response, e
            ))),
        },
        None => Ok(None),
    }
}

fn get_zone(client: &Client, record_name: &str, api_token: &str) -> Result<Zone, UpdateError> {
    let zones: Option<Vec<Zone>> = match make_request(
        client,
        Method::GET,
        "https://api.cloudflare.com/client/v4/zones",
        api_token,
        Some(&[("name.contains", get_sld(record_name))]),
        None::<&UpdateRequest>,
    ) {
        Ok(zones) => zones,
        Err(e) => {
            let message = format!(
                "Failed to get zone for record name: [{}], error: [{:#?}]",
                record_name, e
            );
            return match e {
                UpdateError::Fatal(_) => Err(UpdateError::Fatal(message)),
                UpdateError::Retryable(_) => Err(UpdateError::Retryable(message)),
            };
        }
    };
    let zones = match zones {
        Some(zones) => zones,
        None => {
            return Err(UpdateError::Fatal(format!(
                "Could not find zone for record name: [{}]",
                record_name
            )));
        }
    };
    match get_most_specific_zone(&zones, record_name) {
        Some(zone) => Ok(zone),
        None => Err(UpdateError::Fatal(format!(
            "Could not find zone for record name: [{}]",
            record_name
        ))),
    }
}
fn get_most_specific_zone(zones: &[Zone], record_name: &str) -> Option<Zone> {
    match zones.iter().find(|zone| zone.name == record_name) {
        Some(zone) => Some(zone.clone()),
        None => match record_name.find(".") {
            Some(i) => get_most_specific_zone(zones, &record_name[i + 1..]),
            None => None,
        },
    }
}
fn find_record(
    client: &Client,
    record_name: &str,
    zone_id: &str,
    api_token: &str,
    record_type: &str,
) -> Result<Option<RecordResponse>, UpdateError> {
    let records: Option<Vec<RecordResponse>> = match make_request(
        client,
        Method::GET,
        format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        ),
        api_token,
        Some(&[("name.exact", record_name), ("type", record_type)]),
        None::<&UpdateRequest>,
    ) {
        Ok(records) => records,
        Err(e) => {
            let message = format!(
                "Failed to find record name: [{}], error: [{:#?}]",
                record_name, e
            );
            return match e {
                UpdateError::Fatal(_) => Err(UpdateError::Fatal(message)),
                UpdateError::Retryable(_) => Err(UpdateError::Retryable(message)),
            };
        }
    };
    match records {
        Some(records) => {
            if records.len() > 1 {
                warn!(
                    "Found multiple records for name [{}], updating first record in list [{:#?}]",
                    record_name, records
                );
            }
            Ok(records.into_iter().next())
        }
        None => Ok(None),
    }
}
fn create_record(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    ip: &IpAddr,
    record_name: &str,
) -> Result<RecordUpdate, UpdateError> {
    let record: Option<RecordResponse> = match make_request(
        client,
        Method::POST,
        format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        ),
        api_token,
        None::<&UpdateRequest>,
        Some(&get_create_body(ip, record_name)),
    ) {
        Ok(record) => record,
        Err(e) => {
            let message = format!(
                "Failed to create record name: [{}], error: [{:#?}]",
                record_name, e
            );
            return match e {
                UpdateError::Fatal(_) => Err(UpdateError::Fatal(message)),
                UpdateError::Retryable(_) => Err(UpdateError::Retryable(message)),
            };
        }
    };
    match record {
        Some(record) => Ok(RecordUpdate {
            ip: *ip,
            record_name: record_name.to_string(),
            modified_on: record.modified_on.clone(),
        }),
        None => Err(UpdateError::Fatal(format!(
            "Received emtpy response when creating record for zone ID: [{}], name: [{}]",
            zone_id, record_name
        ))),
    }
}
fn update_record(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    record_id: &str,
    ip: &IpAddr,
    record_name: &str,
) -> Result<RecordUpdate, UpdateError> {
    let record: Option<RecordResponse> = match make_request(
        client,
        Method::PATCH,
        format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        ),
        api_token,
        None::<&UpdateRequest>,
        Some(&get_body(ip, record_name)),
    ) {
        Ok(record) => record,
        Err(e) => {
            let message = format!(
                "Failed to update record name: [{}], error: [{:#?}]",
                record_name, e
            );
            return match e {
                UpdateError::Fatal(_) => Err(UpdateError::Fatal(message)),
                UpdateError::Retryable(_) => Err(UpdateError::Retryable(message)),
            };
        }
    };
    match record {
        Some(record) => Ok(RecordUpdate {
            ip: *ip,
            record_name: record_name.to_string(),
            modified_on: record.modified_on.clone(),
        }),
        None => Err(UpdateError::Fatal(format!(
            "Received emtpy response when updating record for zone ID: [{}], record ID: [{}], name: [{}]",
            zone_id, record_id, record_name
        ))),
    }
}
fn get_body(ip: &IpAddr, record_name: &str) -> UpdateRequest {
    let record_type = match ip {
        IpAddr::V4(_) => "A",
        IpAddr::V6(_) => "AAAA",
    };
    UpdateRequest {
        name: record_name.to_string(),
        content: ip.to_string(),
        comment: format!(
            "Updated by rusty_ddns client for Cloudflare at {}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ),
        r#type: record_type.to_string(),
    }
}
fn get_create_body(ip: &IpAddr, record_name: &str) -> CreateRequest {
    let record_type = match ip {
        IpAddr::V4(_) => "A",
        IpAddr::V6(_) => "AAAA",
    };
    CreateRequest {
        name: record_name.to_string(),
        content: ip.to_string(),
        comment: format!(
            "Updated by rusty_ddns client for Cloudflare at {}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ),
        r#type: record_type.to_string(),
        proxied: false,
    }
}
fn update_or_create(
    client: &Client,
    record_name: &str,
    zone_id: &str,
    api_token: &str,
    record_type: &str,
    ip: &IpAddr,
    allow_create: bool,
) -> Result<RecordUpdate, UpdateError> {
    match find_record(client, record_name, zone_id, api_token, record_type)? {
        Some(record) => update_record(client, api_token, zone_id, &record.id, ip, record_name),
        None => match allow_create {
            true => {
                info!(
                    "Could not find existing {} record for name {}, creating new record",
                    record_name, record_name
                );
                create_record(client, api_token, zone_id, ip, record_name)
            }
            false => Err(UpdateError::Fatal(format!(
                "Could not find existing {} record for name {}, and record creation is disabled",
                record_type, record_name
            ))),
        },
    }
}
pub fn handle_update(request: CloudflareUpdateRequest) -> Result<UpdateResponse, UpdateError> {
    let client = Client::new();
    let zone = get_zone(&client, &request.record_name, &request.api_token)?;
    let ipv4_result = match request.ipv4addr {
        Some(ip) => Some(update_or_create(
            &client,
            &request.record_name,
            &zone.id,
            &request.api_token,
            "A",
            &ip,
            request.allow_create,
        )?),
        None => None,
    };
    let ipv6_result = match request.ipv6addr {
        Some(ip) => Some(update_or_create(
            &client,
            &request.record_name,
            &zone.id,
            &request.api_token,
            "AAAA",
            &ip,
            request.allow_create,
        )?),
        None => None,
    };
    Ok(UpdateResponse {
        ipv4_update: ipv4_result,
        ipv6_update: ipv6_result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone(name: &str) -> Zone {
        Zone {
            id: format!("zone-{name}"),
            name: name.to_string(),
        }
    }

    #[test]
    fn selects_most_specific_zone_for_nested_record() {
        let zones = vec![zone("example.com"), zone("sub.example.com")];

        let selected = get_most_specific_zone(&zones, "host.sub.example.com").unwrap();

        assert_eq!(selected.id, "zone-sub.example.com");
        assert_eq!(selected.name, "sub.example.com");
    }

    #[test]
    fn selects_exact_zone_before_parent_zone() {
        let zones = vec![zone("example.com"), zone("host.example.com")];

        let selected = get_most_specific_zone(&zones, "host.example.com").unwrap();

        assert_eq!(selected.id, "zone-host.example.com");
        assert_eq!(selected.name, "host.example.com");
    }

    #[test]
    fn returns_none_when_no_zone_matches_record_name() {
        let zones = vec![zone("example.net"), zone("sub.example.net")];

        assert!(get_most_specific_zone(&zones, "host.example.com").is_none());
    }

    #[test]
    fn builds_ipv4_update_request_body() {
        let ip: IpAddr = "203.0.113.10".parse().unwrap();

        let body = get_body(&ip, "host.example.com");

        assert_eq!(body.name, "host.example.com");
        assert_eq!(body.content, "203.0.113.10");
        assert_eq!(body.r#type, "A");
        assert!(
            body.comment
                .starts_with("Updated by rusty_ddns client for Cloudflare at ")
        );
    }

    #[test]
    fn builds_ipv6_update_request_body() {
        let ip: IpAddr = "2001:db8::10".parse().unwrap();

        let body = get_body(&ip, "host.example.com");

        assert_eq!(body.name, "host.example.com");
        assert_eq!(body.content, "2001:db8::10");
        assert_eq!(body.r#type, "AAAA");
        assert!(
            body.comment
                .starts_with("Updated by rusty_ddns client for Cloudflare at ")
        );
    }

    #[test]
    fn builds_ipv4_create_request_body_with_proxied_disabled() {
        let ip: IpAddr = "203.0.113.10".parse().unwrap();

        let body = get_create_body(&ip, "host.example.com");

        assert_eq!(body.name, "host.example.com");
        assert_eq!(body.content, "203.0.113.10");
        assert_eq!(body.r#type, "A");
        assert!(!body.proxied);
        assert!(
            body.comment
                .starts_with("Updated by rusty_ddns client for Cloudflare at ")
        );
    }

    #[test]
    fn builds_ipv6_create_request_body_with_proxied_disabled() {
        let ip: IpAddr = "2001:db8::10".parse().unwrap();

        let body = get_create_body(&ip, "host.example.com");

        assert_eq!(body.name, "host.example.com");
        assert_eq!(body.content, "2001:db8::10");
        assert_eq!(body.r#type, "AAAA");
        assert!(!body.proxied);
        assert!(
            body.comment
                .starts_with("Updated by rusty_ddns client for Cloudflare at ")
        );
    }
}
