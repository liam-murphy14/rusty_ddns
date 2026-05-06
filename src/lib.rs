mod cloudflare;
pub mod ip;
pub mod update;

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::{ip, update};

    #[test]
    fn exposes_public_module_entry_points() {
        let _: fn() -> Option<IpAddr> = ip::get_ipv4;
        let _: fn() -> Option<IpAddr> = ip::get_ipv6;
        let _: fn(&str) -> String = update::get_sld;
        let _: fn(update::UpdateRequest) -> Result<update::UpdateResponse, update::UpdateError> =
            update::update_record;
    }

    #[test]
    fn constructs_cloudflare_update_requests_through_public_api() {
        let request = update::UpdateRequest::cloudflare(
            "token".to_string(),
            "host.example.com".to_string(),
            Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 10))),
            None,
            false,
        );

        assert!(matches!(request, update::UpdateRequest::Cloudflare(_)));
    }
}
