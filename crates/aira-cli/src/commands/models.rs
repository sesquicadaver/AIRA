//! Local model inventory + compatibility CLI (QUEUE #58–#59).

use std::path::Path;
use std::process::ExitCode;

use aira_csu_model_compatibility::resolve_and_publish;
use aira_csu_model_inventory::{load_latest, scan_and_publish};
use anyhow::Result;

use crate::cli::ModelsCommands;
use crate::support::ensure_init;

pub(crate) fn run(root: &Path, command: ModelsCommands) -> Result<ExitCode> {
    ensure_init(root)?;
    match command {
        ModelsCommands::Scan { dir } => {
            let out = scan_and_publish(root, dir.as_deref()).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("artifact_id {}", out.artifact_id);
            println!("content_hash {}", out.content_hash);
            println!("installed {}", out.installed_count);
            println!("status scanned");
            Ok(ExitCode::SUCCESS)
        }
        ModelsCommands::List => {
            let (ptr, payload) = load_latest(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("artifact_id {}", ptr.artifact_id);
            println!("content_hash {}", ptr.content_hash);
            println!("updated_at {}", ptr.updated_at);
            if let Some(arr) = payload.get("installed_models").and_then(|v| v.as_array()) {
                println!("installed {}", arr.len());
                for m in arr {
                    if let Some(s) = m.as_str() {
                        println!("model {s}");
                    }
                }
            } else {
                println!("installed 0");
            }
            Ok(ExitCode::SUCCESS)
        }
        ModelsCommands::Compatible => {
            let out = resolve_and_publish(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("assessed {}", out.rows.len());
            for row in &out.rows {
                println!(
                    "{}\t{}\t{}",
                    row.compatibility.as_str(),
                    row.model_ref,
                    row.reason
                );
                println!("evidence {}", row.evidence_artifact_id);
            }
            println!("summary {}", out.summary_path);
            println!("status compatible");
            Ok(ExitCode::SUCCESS)
        }
    }
}
