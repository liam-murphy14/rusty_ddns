use std::{collections::HashMap, net::IpAddr};

use log::{debug, error, info, trace, warn};
use reqwest::{Error, blocking::Client};
use serde::Deserialize;

use crate::{UpdateError, get_sld};

pub struct CloudflareUpdateRequest {
    pub api_token: String,
    pub record_name: String,
    pub ip: IpAddr,
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
// fn update_record(client: &Client, api_token: &str, zone_id: &str, record_id: &str) {}
pub fn handle_update(request: CloudflareUpdateRequest) -> Result<(), UpdateError> {
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
    // TODO: remove
    info!("Zone: [{:#?}]", zone);
    info!("Record: [{:#?}]", record);
    Ok(())
}
