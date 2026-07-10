//! AIRA CLI — schema validation + bootstrap status.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use aira_schema::{find_repo_root, SchemaRegistry};

#[derive(Parser, Debug)]
#[command(
    name = "aira",
    version,
    about = "AIRA CLI — Problem Statement → Verified Result Artifact"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print bootstrap / runtime status.
    Status,
    /// Schema registry commands.
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
}

#[derive(Subcommand, Debug)]
enum SchemaCommands {
    /// List registered schema `$id` values.
    List {
        /// Override schemas directory (default: <repo>/schemas).
        #[arg(long)]
        schemas_dir: Option<PathBuf>,
    },
    /// Validate a JSON file or the fixture suite.
    Validate {
        /// Schema `$id` or short alias (e.g. `ref`, `object-descriptor`).
        #[arg(long)]
        schema: Option<String>,
        /// JSON instance file.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Validate fixtures under repo (uses fixtures/manifest.json).
        #[arg(long)]
        fixtures: Option<PathBuf>,
        /// Override schemas directory.
        #[arg(long)]
        schemas_dir: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => {
            println!("aira {}", env!("CARGO_PKG_VERSION"));
            println!("status: schema registry ready (Epic 2)");
            println!("runtime: not started");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Schema { command } => match command {
            SchemaCommands::List { schemas_dir } => {
                let reg = load_registry(schemas_dir)?;
                for id in reg.list_ids() {
                    println!("{id}");
                }
                Ok(ExitCode::SUCCESS)
            }
            SchemaCommands::Validate {
                schema,
                file,
                fixtures,
                schemas_dir,
            } => {
                let reg = load_registry(schemas_dir)?;
                if let Some(fixtures_root) = fixtures {
                    let root = if fixtures_root.as_os_str() == "fixtures"
                        || fixtures_root.ends_with("fixtures")
                    {
                        find_repo_root(std::env::current_dir()?)?
                    } else {
                        fixtures_root
                    };
                    let report = reg.validate_fixtures(&root)?;
                    println!(
                        "fixtures: passed={} failed={}",
                        report.passed, report.failed
                    );
                    for f in &report.failures {
                        eprintln!("FAIL: {f}");
                    }
                    if report.failed > 0 {
                        return Ok(ExitCode::FAILURE);
                    }
                    return Ok(ExitCode::SUCCESS);
                }

                let schema = schema.context("--schema is required unless --fixtures is set")?;
                let file = file.context("--file is required unless --fixtures is set")?;
                match reg.validate_file(&schema, &file) {
                    Ok(()) => {
                        println!("OK: {} validates against {schema}", file.display());
                        Ok(ExitCode::SUCCESS)
                    }
                    Err(e) => {
                        eprintln!("FAIL: {e}");
                        Ok(ExitCode::FAILURE)
                    }
                }
            }
        },
    }
}

fn load_registry(schemas_dir: Option<PathBuf>) -> Result<SchemaRegistry> {
    let dir = if let Some(d) = schemas_dir {
        d
    } else {
        let root = find_repo_root(std::env::current_dir()?)
            .or_else(|_| find_repo_root(env!("CARGO_MANIFEST_DIR")).context("locate repo root"))?;
        root.join("schemas")
    };
    if !dir.is_dir() {
        bail!("schemas dir not found: {}", dir.display());
    }
    SchemaRegistry::load(dir)
}
