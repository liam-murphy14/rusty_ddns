use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, InfoLevel};

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
    let args = Cli::parse();
}
