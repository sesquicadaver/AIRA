//! Local CSU registry commands (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_csu::{CsuLifecycleState, CsuManifest, CsuRegistry};
use anyhow::{Context, Result};

use crate::cli::CsuCommands;
use crate::support::{default_csu_registry, load_schema_registry};

pub(crate) fn run(root: &Path, command: CsuCommands) -> Result<ExitCode> {
    match command {
        CsuCommands::List { registry } => {
            let path = registry.unwrap_or_else(|| default_csu_registry(root));
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
            let path = registry.unwrap_or_else(|| default_csu_registry(root));
            let mut reg = if path.exists() {
                CsuRegistry::load(&path).map_err(|e| anyhow::anyhow!("{e}"))?
            } else {
                CsuRegistry::new()
            };
            let text = std::fs::read_to_string(&manifest)
                .with_context(|| format!("read {}", manifest.display()))?;
            let m: CsuManifest = serde_json::from_str(&text)
                .with_context(|| format!("parse {}", manifest.display()))?;
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
    }
}
