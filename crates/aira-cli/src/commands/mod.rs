//! CLI command handlers (QUEUE #46 / Analyze-81).

mod conformance;
mod csu;
mod federation;
mod identity;
mod node;
mod peer;
mod problem;
mod schema;
pub(crate) mod tenant;
pub(crate) mod trust;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::cli::Commands;
use crate::support::ensure_init;

pub(crate) fn dispatch(root: PathBuf, command: Commands) -> Result<ExitCode> {
    match command {
        Commands::Init => node::init(&root),
        Commands::Status => node::status(&root),
        Commands::Identity { command } => identity::run(&root, command),
        Commands::Schema { command } => schema::run(command),
        Commands::Csu { command } => csu::run(&root, command),
        Commands::Problem { command } => problem::problem(&root, command),
        Commands::Result { command } => problem::result(&root, command),
        Commands::Artifact { command } => problem::artifact(&root, command),
        Commands::Event { command } => problem::event(&root, command),
        Commands::Peer { command } => {
            ensure_init(&root)?;
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("tokio runtime")?;
            rt.block_on(peer::run_peer(&root, command))
        }
        Commands::Federation { command } => federation::run(&root, command),
        Commands::Conformance { command } => conformance::run(&root, command),
    }
}
