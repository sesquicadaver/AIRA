//! Federation join (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::cli::FederationCommands;
use crate::support::ensure_init;

pub(crate) fn run(root: &Path, command: FederationCommands) -> Result<ExitCode> {
    match command {
        FederationCommands::Join { descriptor } => {
            ensure_init(root)?;
            let raw = std::fs::read_to_string(&descriptor)
                .with_context(|| format!("read {}", descriptor.display()))?;
            let desc: aira_protocol::FederationDescriptor = serde_json::from_str(&raw)
                .with_context(|| format!("parse {}", descriptor.display()))?;
            let out =
                aira_protocol::join_federation(root, &desc).map_err(|e| anyhow::anyhow!("{e}"))?;
            if out.already_member {
                println!(
                    "already joined {}\t{}",
                    out.membership.federation_id, out.membership.identity_ref
                );
            } else {
                println!(
                    "joined {}\ttrusted {}",
                    out.membership.federation_id, out.membership.identity_ref
                );
            }
            println!(
                "membership {}",
                aira_protocol::membership_path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        FederationCommands::Leave => {
            ensure_init(root)?;
            let out = aira_protocol::leave_federation(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if out.was_member {
                println!("left {}", out.federation_id.unwrap_or_default());
            } else {
                println!("not joined");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
