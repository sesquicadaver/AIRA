//! AIRA local node binary — load config, CSU registry, process local events.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::Parser;

use aira_csu::CsuRegistry;
use aira_flow::{init_node, load_config, LocalSession, SubmitOutcome, DEFAULT_AIRA_ROOT};

#[derive(Parser, Debug)]
#[command(
    name = "aira-node",
    version,
    about = "AIRA local node — load config/CSU and process local events"
)]
struct Args {
    /// Local node root (default: .aira).
    #[arg(long, default_value = DEFAULT_AIRA_ROOT)]
    root: PathBuf,

    /// Create layout if missing.
    #[arg(long)]
    init: bool,

    /// Process one problem statement then exit.
    #[arg(long)]
    text: Option<String>,
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
    let args = Args::parse();
    if args.init || !args.root.join("config.json").exists() {
        if args.init {
            let paths = init_node(&args.root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("initialized {}", paths.root.display());
        } else {
            bail!(
                "node not initialized at {} — pass --init or run `aira init`",
                args.root.display()
            );
        }
    }

    let config = load_config(&args.root).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("aira-node {}", env!("CARGO_PKG_VERSION"));
    println!(
        "config: mode={} profile={}",
        config.node.mode, config.node.profile
    );
    println!("autoload: {}", config.csu.autoload.join(", "));

    let registry_path = args.root.join("csu").join("registry.json");
    if registry_path.exists() {
        let reg = CsuRegistry::load(&registry_path).map_err(|e| anyhow::anyhow!("{e}"))?;
        let entries = reg.list();
        println!(
            "csu_registry: {} entries at {}",
            entries.len(),
            registry_path.display()
        );
        for e in entries {
            println!("  {}\t{:?}", e.manifest.csu_id, e.state);
        }
    } else {
        println!("csu_registry: (empty) — built-in basic CSUs used by OperationalPlane");
    }

    if let Some(text) = args.text {
        let mut session = LocalSession::open(&args.root).map_err(|e| anyhow::anyhow!("{e}"))?;
        let out = session
            .submit_problem(&text)
            .map_err(|e| anyhow::anyhow!("process: {e}"))?;
        match out {
            SubmitOutcome::Completed {
                problem_id,
                verified_artifact_id,
                result,
            } => {
                println!("processed problem_ref={problem_id}");
                println!("result_ref={verified_artifact_id}");
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            SubmitOutcome::NeedsHumanCollapse { field_artifact_id } => {
                println!("needs_human_collapse field_ref={field_artifact_id}");
            }
        }
        let tail = session.event_tail(10).map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("event_tail ({})", tail.len());
        for e in tail {
            println!("  {}\t{:?}", e.event_id, e.event_type);
        }
    } else {
        println!("idle: pass --text \"Calculate 2 + 2\" to process a local problem");
    }

    Ok(ExitCode::SUCCESS)
}
