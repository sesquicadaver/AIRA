use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

use crate::cli::PeerDhtCommands;

pub(super) async fn run(root: &Path, command: PeerDhtCommands) -> Result<ExitCode> {
    match command {
        PeerDhtCommands::Announce { addr, from_stun } => {
            let addr = aira_peer::resolve_dht_announce_addr(root, addr.as_deref(), from_stun)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let results = aira_peer::dht_announce_to_peers(root, &addr)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("dht announced {addr}");
            println!("dht {}", aira_peer::PeerDhtStore::path(root).display());
            for (peer, ok, err) in results {
                if ok {
                    println!("announce -> {peer}");
                } else {
                    eprintln!("announce failed {peer}\t{}", err.unwrap_or_default());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        PeerDhtCommands::Find {
            key_ref,
            k,
            apply_book,
        } => {
            let store = aira_peer::PeerDhtStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let exact = store.get(&key_ref).cloned();
            if let Some(ref exact) = exact {
                println!(
                    "exact\t{}\t{}\t{}",
                    exact.identity_id, exact.addr, exact.key_hex
                );
            }
            let closest = store.closest(&key_ref, k);
            if closest.is_empty() {
                println!("(empty dht)");
            } else {
                for r in closest {
                    println!("{}\t{}\t{}", r.identity_id, r.addr, r.key_hex);
                }
            }
            if apply_book {
                match aira_peer::apply_book_exact_from_dht_find(root, &key_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                {
                    Some((id, addr)) => {
                        println!("apply_book {id}\t{addr}");
                        println!(
                            "address_book {}",
                            aira_peer::AddressBook::path(root).display()
                        );
                    }
                    None => println!("apply_book skipped (no exact hit)"),
                }
            }
            println!("dht {}", aira_peer::PeerDhtStore::path(root).display());
            Ok(ExitCode::SUCCESS)
        }
        PeerDhtCommands::List => {
            let store = aira_peer::PeerDhtStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if store.records.is_empty() {
                println!("(empty dht)");
            } else {
                for r in &store.records {
                    let src = r.source.as_deref().unwrap_or("-");
                    println!(
                        "{}\t{}\t{}\t{}\t{}",
                        r.identity_id, r.addr, r.key_hex, r.updated_at, src
                    );
                }
            }
            println!("dht {}", aira_peer::PeerDhtStore::path(root).display());
            Ok(ExitCode::SUCCESS)
        }
    }
}
