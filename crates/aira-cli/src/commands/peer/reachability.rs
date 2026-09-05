//! Phase N `#243`: `aira peer reachability …`

use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::ExitCode;

use aira_object::Keyring;
use anyhow::{bail, Context, Result};

use crate::cli::PeerReachabilityCommands;

pub(super) async fn run(root: &Path, command: PeerReachabilityCommands) -> Result<ExitCode> {
    match command {
        PeerReachabilityCommands::Status => {
            let st = aira_peer::ReachabilityLocalState::load(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("status {:?}", st.status);
            if let Some(p) = st.local_port {
                println!("local_port {p}");
            }
            if let Some(ep) = &st.observed_endpoint {
                println!("observed_endpoint {ep}");
            }
            if let Some(ep) = &st.verified_endpoint {
                println!("verified_endpoint {ep}");
            }
            if let Some(at) = &st.checked_at {
                println!("checked_at {at}");
            }
            if let Some(ev) = &st.probe_evidence {
                println!("probe_evidence {ev}");
            }
            println!("relay_routes {}", st.relay_routes.len());
            println!("may_advertise_direct {}", st.status.may_advertise_direct());
            println!(
                "reachability {}",
                aira_peer::ReachabilityLocalState::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerReachabilityCommands::Check {
            host,
            port,
            result_json,
            mark_direct_failed,
            outbound_ok,
        } => {
            let (id, _) = Keyring::load_node_identity(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut st = aira_peer::ReachabilityLocalState::load(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let now = aira_peer::presence_now().map_err(|e| anyhow::anyhow!("{e}"))?;

            if let Some(path) = result_json {
                let raw = fs::read_to_string(&path)
                    .with_context(|| format!("read result_json {path}"))?;
                let result: aira_peer::ReachabilityResult =
                    serde_json::from_str(&raw).context("parse ReachabilityResult JSON")?;
                st.apply_successful_probe(&result)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                st.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("status {:?}", st.status);
                println!("applied_probe {}", path);
                println!(
                    "reachability {}",
                    aira_peer::ReachabilityLocalState::path(root).display()
                );
                return Ok(ExitCode::SUCCESS);
            }

            if mark_direct_failed {
                st.apply_direct_failed(now, st.relay_routes.clone(), outbound_ok)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                st.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("status {:?}", st.status);
                println!(
                    "reachability {}",
                    aira_peer::ReachabilityLocalState::path(root).display()
                );
                return Ok(ExitCode::SUCCESS);
            }

            let port = port.unwrap_or_else(|| {
                aira_peer::preferred_port(id.as_str(), aira_peer::TransportClass::TcpPeer)
            });
            if !aira_peer::is_valid_aira_port(port) {
                bail!(
                    "port {port} not in P_AIRA — run `aira peer port select` for a prime candidate"
                );
            }
            let bind = format!("{host}:{port}");
            aira_peer::validate_aira_bind(&bind).map_err(|e| anyhow::anyhow!("{e}"))?;
            let listener = TcpListener::bind(&bind)
                .with_context(|| format!("local bind check failed for {bind}"))?;
            drop(listener);
            st.mark_local_bind(port, now)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            st.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("identity {}", id.as_str());
            println!("checked_bind {bind}");
            println!("status {:?}", st.status);
            println!(
                "note LOCAL_ONLY is not DIRECT — peer-assisted probe required for DIRECT_REACHABLE"
            );
            println!(
                "reachability {}",
                aira_peer::ReachabilityLocalState::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
