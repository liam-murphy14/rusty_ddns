use crate::cloudflare::{CloudflareUpdateRequest, handle_update};

pub mod cloudflare;

enum UpdateRequest {
    Cloudflare(CloudflareUpdateRequest),
}
pub enum UpdateError {
    Retryable(String),
    Fatal(String),
}

fn update_record(request: &UpdateRequest) -> Result<(), UpdateError> {
    match request {
        UpdateRequest::Cloudflare(request) => handle_update(request),
    }
}
