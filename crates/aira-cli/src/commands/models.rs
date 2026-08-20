//! Local model inventory / compatibility / acquisition CLI (QUEUE #58–#62).

use std::path::Path;
use std::process::ExitCode;

use aira_csu_model_acquisition::{
    fetch_to_quarantine, load_policy, request_download, write_default_deny_policy, FetchOutcome,
    GateDecision,
};
use aira_csu_model_compatibility::resolve_and_publish;
use aira_csu_model_inventory::{load_latest, scan_and_publish};
use anyhow::Result;

use crate::cli::{ModelsCommands, ModelsPolicyCommands};
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
        ModelsCommands::Policy { command } => match command {
            ModelsPolicyCommands::Show => {
                match load_policy(root).map_err(|e| anyhow::anyhow!("{e}"))? {
                    None => {
                        println!("policy_present false");
                        println!("auto_download false");
                        println!("posture default-deny");
                    }
                    Some(p) => {
                        println!("policy_present true");
                        println!("auto_download {}", p.auto_download);
                        println!("allow_untrusted_models {}", p.allow_untrusted_models);
                        println!("share_custom_models {}", p.share_custom_models);
                    }
                }
                Ok(ExitCode::SUCCESS)
            }
            ModelsPolicyCommands::Set { auto_download } => {
                let path = write_default_deny_policy(root, auto_download)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("policy {}", path.display());
                println!("auto_download {auto_download}");
                println!("status policy-set");
                Ok(ExitCode::SUCCESS)
            }
        },
        ModelsCommands::Download { model_ref, source } => {
            if let Some(src) = source {
                let out = fetch_to_quarantine(root, &model_ref, &src)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                match out {
                    FetchOutcome::Denied(gate) => {
                        println!("decision {}", gate.decision.as_str());
                        println!("model_ref {}", gate.model_ref);
                        println!("reason {}", gate.reason);
                        println!("reason_ref {}", gate.reason_ref);
                        println!("evidence {}", gate.decision_artifact_id);
                        println!("status policy-denied");
                        Ok(ExitCode::from(2))
                    }
                    FetchOutcome::Quarantined {
                        gate,
                        quarantine_path,
                        bytes,
                        content_hash,
                        source_path,
                    } => {
                        println!("decision {}", gate.decision.as_str());
                        println!("model_ref {}", gate.model_ref);
                        println!("reason {}", gate.reason);
                        println!("evidence {}", gate.decision_artifact_id);
                        println!("source {source_path}");
                        println!("quarantine {quarantine_path}");
                        println!("bytes {bytes}");
                        println!("content_hash {content_hash}");
                        println!("verified false");
                        println!("activated false");
                        println!("status quarantine-fetched");
                        Ok(ExitCode::SUCCESS)
                    }
                }
            } else {
                let out = request_download(root, &model_ref).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("decision {}", out.decision.as_str());
                println!("model_ref {}", out.model_ref);
                println!("reason {}", out.reason);
                println!("reason_ref {}", out.reason_ref);
                println!("evidence {}", out.decision_artifact_id);
                match out.decision {
                    GateDecision::Allow => {
                        println!("status policy-allowed");
                        Ok(ExitCode::SUCCESS)
                    }
                    GateDecision::Deny => {
                        println!("status policy-denied");
                        Ok(ExitCode::from(2))
                    }
                }
            }
        }
    }
}
