use crate::cloudflare::{CloudflareUpdateRequest, handle_update};

pub mod cloudflare;

pub enum UpdateRequest {
    Cloudflare(CloudflareUpdateRequest),
}
pub enum UpdateError {
    Retryable(String),
    Fatal(String),
}

pub fn update_record(request: UpdateRequest) -> Result<(), UpdateError> {
    match request {
        UpdateRequest::Cloudflare(request) => handle_update(request),
    }
}
