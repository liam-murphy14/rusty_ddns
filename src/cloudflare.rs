use std::{fmt::format, net::IpAddr};

use crate::UpdateError;

pub struct CloudflareUpdateRequest {
    pub api_token: String,
    pub record_name: String,
    pub ip: IpAddr,
}

pub fn handle_update(request: &CloudflareUpdateRequest) -> Result<(), UpdateError> {
    let client = reqwest::blocking::Client::new();
    let existing_record = client.get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", request.api_token))
        .query(&[("name", &request.record_name)]);
    println!("{}", request.ip);
    Ok(())
}
