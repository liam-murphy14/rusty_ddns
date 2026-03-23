use std::net::{IpAddr, Ipv4Addr};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{error, info};
use rusty_ddns::{UpdateRequest, cloudflare::CloudflareUpdateRequest, update_record};

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
    },
}

fn main() {
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(cli.verbosity.log_level_filter())
        .init();

    match cli.command {
        Commands::Cloudflare {
            api_token,
            record_name,
        } => {
            match update_record(UpdateRequest::Cloudflare(CloudflareUpdateRequest {
                api_token,
                record_name,
                ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                allow_create: true,
            })) {
                Ok(()) => info!("ok record"),
                Err(error) => error!("error: [{:#?}]", error),
            };
        }
    }
}
