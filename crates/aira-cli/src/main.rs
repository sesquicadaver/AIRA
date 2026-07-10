//! AIRA CLI — schema validation + CSU registry + bootstrap status.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use aira_csu::{CsuLifecycleState, CsuManifest, CsuRegistry};
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
    /// Local CSU registry commands.
    Csu {
        #[command(subcommand)]
        command: CsuCommands,
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

#[derive(Subcommand, Debug)]
enum CsuCommands {
    /// List registered CSU from the local registry file.
    List {
        /// Registry JSON path (default: .aira/csu-registry.json).
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Register a CSU manifest into the local registry.
    Register {
        /// Path to CSU manifest JSON.
        #[arg(long)]
        manifest: PathBuf,
        /// Registry JSON path (default: .aira/csu-registry.json).
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Activate after register (Registered→Verified→Active).
        #[arg(long)]
        activate: bool,
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
            println!("status: C1 CSU Runtime ready (Epic 5)");
            println!("runtime: local CSU registry available (`aira csu list`)");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Schema { command } => match command {
            SchemaCommands::List { schemas_dir } => {
                let reg = load_schema_registry(schemas_dir)?;
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
                let reg = load_schema_registry(schemas_dir)?;
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
        Commands::Csu { command } => match command {
            CsuCommands::List { registry } => {
                let path = registry.unwrap_or_else(default_csu_registry_path);
                if !path.exists() {
                    println!("(empty) no registry at {}", path.display());
                    return Ok(ExitCode::SUCCESS);
                }
                let reg = CsuRegistry::load(&path).map_err(|e| anyhow::anyhow!("{e}"))?;
                for entry in reg.list() {
                    println!(
                        "{}\t{:?}\t{:?}\t{}",
                        entry.manifest.csu_id,
                        entry.state,
                        entry.manifest.csu_type,
                        entry.manifest.csu_name
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
            CsuCommands::Register {
                manifest,
                registry,
                activate,
            } => {
                let path = registry.unwrap_or_else(default_csu_registry_path);
                let mut reg = if path.exists() {
                    CsuRegistry::load(&path).map_err(|e| anyhow::anyhow!("{e}"))?
                } else {
                    CsuRegistry::new()
                };
                let text = std::fs::read_to_string(&manifest)
                    .with_context(|| format!("read {}", manifest.display()))?;
                let m: CsuManifest = serde_json::from_str(&text)
                    .with_context(|| format!("parse {}", manifest.display()))?;
                // Schema validate when possible.
                if let Ok(schema_reg) = load_schema_registry(None) {
                    let v = serde_json::to_value(&m)?;
                    schema_reg
                        .validate("aira:schema:csu:manifest:0.1", &v)
                        .map_err(|e| anyhow::anyhow!("manifest schema: {e}"))?;
                }
                let id = m.csu_id.clone();
                reg.register(m, None)
                    .map_err(|e| anyhow::anyhow!("register: {e}"))?;
                if activate {
                    reg.activate(&id, None)
                        .map_err(|e| anyhow::anyhow!("activate: {e}"))?;
                }
                reg.save(&path)
                    .map_err(|e| anyhow::anyhow!("save registry: {e}"))?;
                let state = reg
                    .get(&id)
                    .map(|e| e.state)
                    .unwrap_or(CsuLifecycleState::Registered);
                println!("registered {} state={state:?}", id);
                println!("registry {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
        },
    }
}

fn default_csu_registry_path() -> PathBuf {
    PathBuf::from(".aira/csu-registry.json")
}

fn load_schema_registry(schemas_dir: Option<PathBuf>) -> Result<SchemaRegistry> {
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
