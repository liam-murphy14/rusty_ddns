use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{error, info};
use rusty_ddns::update::{UpdateRequest, update_record};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// DDNS client program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Control output level
    #[command(flatten)]
    verbosity: Verbosity<WarnLevel>,

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

    // TODO: parse IP address from system or remote call

    match cli.command {
        Commands::Cloudflare {
            api_token,
            record_name,
            allow_create,
        } => {
            match update_record(UpdateRequest::cloudflare(
                api_token,
                record_name,
                Some(IpAddr::V4(
                    Ipv4Addr::from_str("24.18.226.122").expect("test"),
                )),
                Some(IpAddr::V6(
                    Ipv6Addr::from_str("2601:602:9601:a690:8ba4:4bdf:ebc1:3d35").expect("test"),
                )),
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
