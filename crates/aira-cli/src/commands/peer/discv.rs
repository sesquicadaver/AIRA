use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::cli::PeerDiscvCommands;

pub(super) async fn run(root: &Path, command: PeerDiscvCommands) -> Result<ExitCode> {
    match command {
        PeerDiscvCommands::Listen {
            bind,
            once,
            explicit,
        } => {
            let sock = if explicit {
                aira_peer::bind_udp_explicit(&bind)
            } else {
                aira_peer::bind_udp(&bind)
            }
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("discv listening {}", sock.local_addr()?);
            loop {
                match aira_peer::recv_one_and_handle(&sock, root) {
                    Ok(aira_peer::DiscvHandleResult::StoredAnnounce(a)) => {
                        println!("discv stored {}\t{}", a.identity_id, a.addr);
                        println!("dht {}", aira_peer::PeerDhtStore::path(root).display());
                        if once {
                            return Ok(ExitCode::SUCCESS);
                        }
                    }
                    Ok(aira_peer::DiscvHandleResult::AnsweredFind {
                        requester,
                        target_id,
                        n,
                    }) => {
                        println!("discv nodes {requester} target {target_id} n={n}");
                        if once {
                            return Ok(ExitCode::SUCCESS);
                        }
                    }
                    Err(e) => {
                        eprintln!("discv drop: {e}");
                        if once {
                            return Err(anyhow::anyhow!("{e}"));
                        }
                    }
                }
            }
        }
        PeerDiscvCommands::Announce {
            to,
            addr,
            from_stun,
        } => {
            let to: std::net::SocketAddr =
                to.parse().with_context(|| format!("invalid --to {to}"))?;
            let advertised = aira_peer::resolve_dht_announce_addr(root, addr.as_deref(), from_stun)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            aira_peer::send_discv_announce(root, &advertised, to)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("discv announced {advertised} -> {to}");
            Ok(ExitCode::SUCCESS)
        }
        PeerDiscvCommands::Find { key_ref, to, k } => {
            let mut seeds = Vec::new();
            if let Some(to) = to {
                seeds.push(
                    to.parse::<std::net::SocketAddr>()
                        .with_context(|| format!("invalid --to {to}"))?,
                );
            }
            let report = aira_peer::iterative_discv_find(root, &key_ref, &seeds, k)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!(
                "discv find hops={} queried={} stored={}",
                report.hops, report.queried, report.stored
            );
            match report.exact {
                Some((id, addr)) => println!("exact\t{id}\t{addr}"),
                None => println!("exact\t(none)"),
            }
            println!("dht {}", aira_peer::PeerDhtStore::path(root).display());
            Ok(ExitCode::SUCCESS)
        }
    }
}
