//! AIRA CLI skeleton (MVP bootstrap).

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "aira",
    version,
    about = "AIRA CLI skeleton — Problem Statement → Verified Result Artifact"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print bootstrap status (no runtime yet).
    Status,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => {
            println!("aira {}", env!("CARGO_PKG_VERSION"));
            println!("status: workspace bootstrap (Epic 0)");
            println!("runtime: not started");
        }
    }
}
