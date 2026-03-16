use std::net::{IpAddr, Ipv4Addr};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, InfoLevel};
use rusty_ddns::{UpdateRequest, cloudflare::CloudflareUpdateRequest, update_record};

/// DDNS client program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Control output level
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

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
}
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Cloudflare { api_token, record_name } => {
            let _ = update_record(UpdateRequest::Cloudflare(CloudflareUpdateRequest { api_token, record_name, ip: IpAddr::V4(Ipv4Addr::new(127,0,0,1))}));
        }
    }
}
