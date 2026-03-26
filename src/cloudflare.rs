use log::{debug, error, info, trace, warn};
use reqwest::{Error, blocking::Client};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use crate::{RecordUpdate, UpdateError, UpdateResponse, get_sld};

pub struct CloudflareUpdateRequest {
    pub api_token: String,
    pub record_name: String,
    pub ipv4addr: IpAddr,
    pub ipv6addr: Option<IpAddr>,
    pub allow_create: bool,
}

#[derive(Deserialize, Debug)]
struct GetRecordsResponse {
    pub errors: Vec<ResponseInfo>,
    pub messages: Vec<ResponseInfo>,
    pub success: bool,
    pub result: Option<Vec<RecordResponse>>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Deserialize, Debug)]
struct RecordResponse {
    pub id: String,
    pub created_on: String,
    pub modified_on: String,
}

#[derive(Deserialize, Debug)]
struct GetZonesResponse {
    pub errors: Vec<ResponseInfo>,
    pub messages: Vec<ResponseInfo>,
    pub success: bool,
    pub result: Option<Vec<Zone>>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Deserialize, Debug, Clone)]
struct Zone {
    pub id: String,
    pub name: String,
    pub account: Account,
}

#[derive(Deserialize, Debug, Clone)]
struct Account {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct ResultInfo {
    pub count: Option<u32>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub total_count: Option<u32>,
    pub total_pages: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct ResponseInfo {
    pub code: u32,
    pub message: String,
    pub documentation_url: Option<String>,
    pub source: Option<Source>,
}

#[derive(Serialize, Debug)]
struct UpdateRequest {
    pub name: String,
    pub content: String,
    pub comment: String,
    pub r#type: String,
}

#[derive(Serialize, Debug)]
struct CreateRequest {
    pub name: String,
    pub content: String,
    pub comment: String,
    pub r#type: String,
    pub proxied: bool,
}

#[derive(Deserialize, Debug)]
struct Source {
    pub pointer: Option<String>,
}

fn get_zone(client: &Client, record_name: &str, api_token: &str) -> Result<Option<Zone>, Error> {
    trace!("In get_zone, arguments - record_name: [{:#?}]", record_name);
    let existing_record_res: GetZonesResponse = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", api_token))
        .query(&[("name.contains", get_sld(record_name))])
        .send()?
        .json()?;
    debug!(
        "Received response from GetZones CloudFlare API, res: [{:#?}]",
        existing_record_res
    );

    if !existing_record_res.success {
        error!(
            "Received unsuccessful response from GetZones CloudFlare API, errors: [{:#?}]",
            existing_record_res.errors
        );
        return Ok(None);
    } else if existing_record_res.errors.len() > 0 {
        warn!(
            "Received errors from GetZones CloudFlare API, errors: [{:#?}]",
            existing_record_res.errors
        )
    }

    if existing_record_res.messages.len() > 0 {
        info!(
            "Received messages from GetZones CloudFlare API, messages: [{:#?}]",
            existing_record_res.messages
        );
    }

    match existing_record_res.result {
        Some(zones) => Ok(get_most_specific_zone(&zones, record_name)),
        None => Ok(None),
    }
}
fn get_most_specific_zone(zones: &[Zone], record_name: &str) -> Option<Zone> {
    match zones.into_iter().find(|zone| zone.name == record_name) {
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
) -> Result<Option<RecordResponse>, Error> {
    trace!(
        "In find_record, arguments - record_name: [{:#?}], zone_id: [{:#?}]",
        record_name, zone_id
    );
    let existing_record_res: GetRecordsResponse = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        ))
        .header("Authorization", format!("Bearer {}", api_token))
        .query(&[("name.exact", record_name)])
        .send()?
        .json()?;
    debug!(
        "Received response from GetDnsRecords CloudFlare API, input zone ID: [{:#?}], res: [{:#?}]",
        zone_id, existing_record_res
    );

    if !existing_record_res.success {
        error!(
            "Received unsuccessful response from GetDnsRecords CloudFlare API, input zone ID: [{:#?}], errors: [{:#?}]",
            zone_id, existing_record_res.errors
        );
        return Ok(None);
    } else if existing_record_res.errors.len() > 0 {
        warn!(
            "Received errors from GetDnsRecords CloudFlare API, errors: [{:#?}]",
            existing_record_res.errors
        )
    }

    if existing_record_res.messages.len() > 0 {
        info!(
            "Received messages from GetZones CloudFlare API, messages: [{:#?}]",
            existing_record_res.messages
        );
    }

    match existing_record_res.result {
        Some(records) => Ok(records.into_iter().next()),
        None => Ok(None),
    }
}
fn update_record(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    record_id: &str,
    ip: &IpAddr,
    record_name: &str,
) -> Result<RecordUpdate, Error> {
    trace!(
        "In update_record, arguments - zone_id: [{:#?}], record_id: [{:#?}]",
        zone_id, record_id
    );
    let _update_record_response: GetRecordsResponse = client
        .patch(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        ))
        .header("Authorization", format!("Bearer {}", api_token))
        .json(&get_body(ip, record_name))
        .send()?
        .json()?;
    Ok(RecordUpdate {
        ip: ip.clone(),
        record_name: record_name.to_string(),
    })
}
fn get_body(ip: &IpAddr, record_name: &str) -> UpdateRequest {
    let record_type = match ip {
        IpAddr::V4(_) => "A",
        IpAddr::V6(_) => "AAAA",
    };
    UpdateRequest {
        name: record_name.to_string(),
        content: ip.to_string(),
        comment: String::from("Updated by rusty_ddns client for Cloudflare"),
        r#type: record_type.to_string(),
    }
}
pub fn handle_update(request: CloudflareUpdateRequest) -> Result<UpdateResponse, UpdateError> {
    let client = Client::new();
    let zone = match get_zone(&client, &request.record_name, &request.api_token) {
        Ok(Some(zone)) => zone,
        Ok(None) => {
            return Err(UpdateError::Retryable(String::from(
                "Could not find existing record",
            )));
        }
        Err(error) => {
            return Err(UpdateError::Retryable(String::from(format!(
                "Failed to fetch existing record, error [{:#?}]",
                error
            ))));
        }
    };
    let record = match find_record(&client, &request.record_name, &zone.id, &request.api_token) {
        Ok(Some(record)) => record,
        // TODO: update this to create a new record
        Ok(None) => {
            return Err(UpdateError::Retryable(String::from(
                "Could not find existing record",
            )));
        }
        Err(error) => {
            return Err(UpdateError::Retryable(String::from(format!(
                "Failed to fetch existing record, error [{:#?}]",
                error
            ))));
        }
    };
    let ipv4_result = match update_record(
        &client,
        &request.api_token,
        &zone.id,
        &record.id,
        &request.ipv4addr,
        &request.record_name,
    ) {
        Ok(result) => result,
        Err(_e) => {
            return Err(UpdateError::Retryable(String::from(
                "Could not update ipv4 address",
            )));
        }
    };
    if let Some(ip) = request.ipv6addr {
        let ipv6_result = match update_record(
            &client,
            &request.api_token,
            &zone.id,
            &record.id,
            &ip,
            &request.record_name,
        ) {
            Ok(result) => Some(result),
            Err(_e) => {
                warn!("Could not update ipv6 record, ignoring");
                None
            }
        };
        return Ok(UpdateResponse {
            ipv4_update: ipv4_result,
            ipv6_update: ipv6_result,
        });
    }
    Ok(UpdateResponse {
        ipv4_update: ipv4_result,
        ipv6_update: None,
    })
}
