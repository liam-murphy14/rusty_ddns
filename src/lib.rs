use crate::cloudflare::{CloudflareUpdateRequest, handle_update};
use reqwest::Error;
use std::net::IpAddr;

pub mod cloudflare;

pub enum UpdateRequest {
    Cloudflare(CloudflareUpdateRequest),
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
