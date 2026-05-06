use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{error, info};
use rusty_ddns::ip::{get_ipv4, get_ipv6};
use rusty_ddns::update::{UpdateRequest, update_record};
use std::net::IpAddr;

/// DDNS client program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Control output level
    #[command(flatten)]
    verbosity: Verbosity<WarnLevel>,

    /// Only create record for IPv4
    #[arg(short = '4', long = "4", action = clap::ArgAction::SetTrue)]
    only_four: bool,

    /// Only create record for IPv6
    #[arg(short = '6', long = "6", action = clap::ArgAction::SetTrue)]
    only_six: bool,

    /// Subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Cloudflare {
        #[arg(short, long)]
        api_token: String,
        #[arg(short, long)]
        record_name: String,
        #[arg(long)]
        allow_create: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    let mut v4: Option<IpAddr> = None;
    let mut v6: Option<IpAddr> = None;
    if cli.only_four && !cli.only_six {
        v4 = get_ipv4();
    } else if !cli.only_four && cli.only_six {
        v6 = get_ipv6();
    } else {
        v4 = get_ipv4();
        v6 = get_ipv6();
    }

    if let (None, None) = (v4, v6) {
        error!("Neither v4 nor v6 IP address could be found.");
        return;
    }

    match cli.command {
        Commands::Cloudflare {
            api_token,
            record_name,
            allow_create,
        } => {
            match update_record(UpdateRequest::cloudflare(
                api_token,
                record_name,
                v4,
                v6,
                allow_create,
            )) {
                Ok(response) => info!("Successfully updated Cloudflare records: [{:#?}]", response),
                Err(error) => error!(
                    "Unexpected error occurred while attempting to update Cloudflare records: [{:#?}]",
                    error
                ),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_cloudflare(args: &[&str]) -> Cli {
        let mut argv = vec!["rusty_ddns"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).expect("CLI arguments should parse")
    }

    fn assert_cloudflare_command(cli: Cli, expected_allow_create: bool) {
        match cli.command {
            Commands::Cloudflare {
                api_token,
                record_name,
                allow_create,
            } => {
                assert_eq!(api_token, "token");
                assert_eq!(record_name, "host.example.com");
                assert_eq!(allow_create, expected_allow_create);
            }
        }
    }

    #[test]
    fn parses_ipv4_only_flag() {
        let cli = parse_cloudflare(&[
            "--4",
            "cloudflare",
            "--api-token",
            "token",
            "--record-name",
            "host.example.com",
        ]);

        assert!(cli.only_four);
        assert!(!cli.only_six);
        assert_cloudflare_command(cli, false);
    }

    #[test]
    fn parses_ipv6_only_flag() {
        let cli = parse_cloudflare(&[
            "--6",
            "cloudflare",
            "--api-token",
            "token",
            "--record-name",
            "host.example.com",
        ]);

        assert!(!cli.only_four);
        assert!(cli.only_six);
        assert_cloudflare_command(cli, false);
    }

    #[test]
    fn defaults_to_both_ip_families_when_no_family_flag_is_set() {
        let cli = parse_cloudflare(&[
            "cloudflare",
            "--api-token",
            "token",
            "--record-name",
            "host.example.com",
        ]);

        assert!(!cli.only_four);
        assert!(!cli.only_six);
        assert_cloudflare_command(cli, false);
    }

    #[test]
    fn accepts_both_ip_family_flags() {
        let cli = parse_cloudflare(&[
            "--4",
            "--6",
            "cloudflare",
            "--api-token",
            "token",
            "--record-name",
            "host.example.com",
        ]);

        assert!(cli.only_four);
        assert!(cli.only_six);
        assert_cloudflare_command(cli, false);
    }

    #[test]
    fn parses_cloudflare_required_arguments() {
        let cli = parse_cloudflare(&[
            "cloudflare",
            "--api-token",
            "token",
            "--record-name",
            "host.example.com",
        ]);

        assert_cloudflare_command(cli, false);
    }

    #[test]
    fn parses_allow_create_flag() {
        let cli = parse_cloudflare(&[
            "cloudflare",
            "--api-token",
            "token",
            "--record-name",
            "host.example.com",
            "--allow-create",
        ]);

        assert_cloudflare_command(cli, true);
    }

    #[test]
    fn rejects_cloudflare_without_api_token() {
        let error = Cli::try_parse_from([
            "rusty_ddns",
            "cloudflare",
            "--record-name",
            "host.example.com",
        ])
        .expect_err("missing API token should be rejected");

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn rejects_cloudflare_without_record_name() {
        let error = Cli::try_parse_from(["rusty_ddns", "cloudflare", "--api-token", "token"])
            .expect_err("missing record name should be rejected");

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }
}
