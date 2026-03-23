use crate::cloudflare::{CloudflareUpdateRequest, handle_update};

pub mod cloudflare;

pub enum UpdateRequest {
    Cloudflare(CloudflareUpdateRequest),
}
#[derive(Debug)]
pub enum UpdateError {
    Retryable(String),
    Fatal(String),
}

pub fn update_record(request: UpdateRequest) -> Result<(), UpdateError> {
    match request {
        UpdateRequest::Cloudflare(request) => handle_update(request),
    }
}

// record name already validated
pub fn get_sld(valid_record_name: &str) -> String {
    let names = valid_record_name.split('.').collect::<Vec<_>>();
    names[names.len() - 2..].join(".")
}
