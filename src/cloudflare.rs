use std::{net::IpAddr};

use log::{debug, error, info, warn};
use reqwest::{Error, blocking::Client};
use serde::Deserialize;

use crate::UpdateError;

pub struct CloudflareUpdateRequest {
    pub api_token: String,
    pub record_name: String,
    pub ip: IpAddr,
}

#[derive(Deserialize, Debug)]
struct GetZonesResponse {
    pub errors: Vec<ResponseInfo>,
    pub messages: Vec<ResponseInfo>,
    pub success: bool,
    pub result: Option<Vec<Zone>>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Deserialize, Debug)]
struct Zone {
    pub id: String,
    pub name: String,
    pub account: Account,
}

#[derive(Deserialize, Debug)]
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
    pub pointer: Option<String>
}

fn get_existing_record(client: &Client, record_name: &str, api_token: &str) -> Result<Option<Zone>, Error> {
    let existing_record_res: GetZonesResponse = client.get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", api_token))
        .query(&[("name", &record_name)])
        .send()?.json()?;
    debug!("Received response from GetZones CloudFlare API, res: [{:#?}]", existing_record_res);

    if !existing_record_res.success {
        error!("Received unsuccessful response from GetZones CloudFlare API, errors: [{:#?}]", existing_record_res.errors);
        return Ok(None);
    } else if existing_record_res.errors.len() > 0 {
        warn!("Received errors from GetZones CloudFlare API, errors: [{:#?}]", existing_record_res.errors)
    }
    info!("Received messages from GetZones CloudFlare API, messages: [{:#?}]", existing_record_res.messages);

    match existing_record_res.result {
        // TODO
        Some(zones) => {
            Ok(zones.into_iter().filter(|zone| zone.name == record_name).next())
        },
        None => Ok(None),
    }
    
    // existing_record_res.mes
    // match existing_record_res {
    //     Ok(res) => {
    //         match res.json()::<GetZonesResponse> {
    //             Ok(data) => {}
    //             Err(err) => {
    //         match err.status() {
    //             Some(status_code) => error!("Failed to get existing record for domain: [{}], status code: [{}], error: [{}]", request.record_name, status_code, err),
    //             None => error!("Failed to get existing record for domain: [{}] without status code, error: [{}]", request.record_name, err)
    //         }
    //             }
    //         }
    //     }
    //     Err(err) => {
    //         match err.status() {
    //             Some(status_code) => error!("Failed to get existing record for domain: [{}], status code: [{}], error: [{}]", request.record_name, status_code, err),
    //             None => error!("Failed to get existing record for domain: [{}] without status code, error: [{}]", request.record_name, err)
    //         }
    //     }
    // }
}
pub fn handle_update(request: &CloudflareUpdateRequest) -> Result<(), UpdateError> {
    let client = Client::new();
    let zone = match get_existing_record(&client, &request.record_name, &request.api_token) {
        Ok(Some(zone)) => zone,
        Ok(None) => return Err(UpdateError::Retryable(String::from("Could not find existing record")))
        Err(error) => return Err(UpdateError::Retryable(String::from("Could not find existing record")))
    };
    Ok(())
}
