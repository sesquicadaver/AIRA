//! Phase N `#243`: `aira peer rendezvous …`

use std::path::Path;
use std::process::ExitCode;

use aira_object::Keyring;
use aira_peer::RendezvousProvider;
use anyhow::{bail, Result};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::cli::PeerRendezvousCommands;

fn expires_after(now: &str, ttl_secs: u64) -> Result<String> {
    let base = OffsetDateTime::parse(now, &Rfc3339)
        .map_err(|e| anyhow::anyhow!("bad now timestamp: {e}"))?;
    Ok((base + Duration::seconds(ttl_secs as i64))
        .format(&Rfc3339)
        .expect("rfc3339"))
}

fn reach_relay_presence(
    reach: &aira_peer::ReachabilityLocalState,
    expires_at: &str,
) -> Vec<aira_peer::PresenceRelayEndpoint> {
    reach
        .relay_routes
        .iter()
        .map(|r| aira_peer::PresenceRelayEndpoint {
            relay_identity_ref: r.relay_identity_ref.clone(),
            relay_endpoint: r.relay_endpoint.clone(),
            reservation_id: r.reservation_id.clone().unwrap_or_else(|| "cli".into()),
            expires_at: expires_at.into(),
        })
        .collect()
}

fn build_signed_presence(
    root: &Path,
    host: &str,
    port: u16,
    ttl_secs: u64,
) -> Result<aira_peer::NodePresenceRecord> {
    let (id, ring) = Keyring::load_node_identity(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    let pk = ring
        .verifying_key(id.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing verifying key"))?;
    let pk_hex = hex::encode(pk.as_bytes());
    let now = aira_peer::presence_now().map_err(|e| anyhow::anyhow!("{e}"))?;
    let expires_at = expires_after(&now, ttl_secs)?;
    let reach =
        aira_peer::ReachabilityLocalState::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    let hint = reach.to_presence_hint();
    let relays = reach_relay_presence(&reach, &expires_at);
    let directs = vec![aira_peer::PresenceDirectEndpoint {
        transport: "tcp-peer".into(),
        host: host.into(),
        port,
        reachability_state: hint,
        observed_at: now.clone(),
    }];

    let provider =
        aira_peer::LocalFileRendezvousProvider::open(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    let existing = provider
        .query_identity(id.as_str())
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let draft = if let Some(prev) = existing {
        let same_endpoints = prev.direct_endpoints == directs && prev.relay_endpoints == relays;
        if same_endpoints {
            aira_peer::refresh_presence_draft(&prev, &now, ttl_secs)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        } else {
            aira_peer::endpoint_change_presence_draft(&prev, &now, ttl_secs, directs, relays)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
    } else {
        aira_peer::PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pk_hex,
            sequence: 1,
            created_at: now,
            expires_at,
            direct_endpoints: directs,
            relay_endpoints: relays,
            capabilities_hash: aira_peer::empty_capabilities_hash(),
        }
    };

    aira_peer::sign_presence_draft_for_node(root, draft).map_err(|e| anyhow::anyhow!("{e}"))
}

pub(super) async fn run(root: &Path, command: PeerRendezvousCommands) -> Result<ExitCode> {
    match command {
        PeerRendezvousCommands::Status => {
            let st =
                aira_peer::RendezvousLocalState::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("provider {}", st.provider);
            println!("network_id {}", st.network_id);
            println!("local_sequence {}", st.local_sequence);
            if let Some(p) = &st.last_publish {
                println!("last_publish {p}");
            }
            if let Some(q) = &st.last_query {
                println!("last_query {q}");
            }
            if let Some(h) = &st.local_presence_hash {
                println!("local_presence_hash {h}");
            }
            println!(
                "rendezvous {}",
                aira_peer::RendezvousLocalState::path(root).display()
            );
            println!(
                "ledger {}",
                aira_peer::LocalFileRendezvousProvider::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerRendezvousCommands::Publish {
            host,
            port,
            ttl_secs,
        } => {
            let (id, _) = Keyring::load_node_identity(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let port = port.unwrap_or_else(|| {
                aira_peer::preferred_port(id.as_str(), aira_peer::TransportClass::TcpPeer)
            });
            if !aira_peer::is_valid_aira_port(port) {
                bail!("advertise port {port} not in P_AIRA");
            }
            let record = build_signed_presence(root, &host, port, ttl_secs)?;
            let mut provider = aira_peer::LocalFileRendezvousProvider::open(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let had = provider
                .query_identity(id.as_str())
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .is_some();
            let mut client = aira_peer::RendezvousClient::new(&mut provider).with_root(root);
            let call = if had {
                client
                    .update_presence(record.clone())
                    .map_err(|e| anyhow::anyhow!("{e}"))?
            } else {
                client
                    .publish_presence(record.clone())
                    .map_err(|e| anyhow::anyhow!("{e}"))?
            };
            println!("identity {}", record.identity_ref);
            println!("sequence {}", record.sequence);
            println!("expires_at {}", record.expires_at);
            println!("identity_hash {}", call.identity_hash);
            println!(
                "ledger {}",
                aira_peer::LocalFileRendezvousProvider::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerRendezvousCommands::Query { as_of, identity } => {
            let as_of = match as_of {
                Some(s) => s,
                None => aira_peer::presence_now().map_err(|e| anyhow::anyhow!("{e}"))?,
            };
            let mut provider = aira_peer::LocalFileRendezvousProvider::open(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut client = aira_peer::RendezvousClient::new(&mut provider).with_root(root);
            if let Some(id) = identity {
                match client
                    .query_identity(&id)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                {
                    Some(rec) => {
                        let active = !aira_peer::is_presence_expired(&rec, &as_of)
                            .map_err(|e| anyhow::anyhow!("{e}"))?;
                        println!(
                            "{}\tseq {}\texpires {}\tactive {}",
                            rec.identity_ref, rec.sequence, rec.expires_at, active
                        );
                    }
                    None => println!("(no record for {id})"),
                }
            } else {
                let active = client
                    .query_active_peers(&as_of)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                if active.is_empty() {
                    println!("(no active peers at {as_of})");
                } else {
                    for rec in active {
                        println!(
                            "{}\tseq {}\texpires {}",
                            rec.identity_ref, rec.sequence, rec.expires_at
                        );
                    }
                }
            }
            println!(
                "ledger {}",
                aira_peer::LocalFileRendezvousProvider::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
