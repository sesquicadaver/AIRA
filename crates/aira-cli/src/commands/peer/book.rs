use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::support::require_trusted;

pub(super) async fn add(
    root: &Path,
    key_ref: String,
    addr: String,
    via: Option<String>,
) -> Result<ExitCode> {
    require_trusted(root, &key_ref)?;
    addr.parse::<std::net::SocketAddr>()
        .with_context(|| format!("invalid addr {addr}"))?;
    if let Some(ref via_id) = via {
        require_trusted(root, via_id)?;
    }
    let mut book = aira_peer::AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    book.upsert_via(&key_ref, &addr, via.clone());
    book.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    match &via {
        Some(v) => println!("peer {key_ref} -> {addr} via {v}"),
        None => println!("peer {key_ref} -> {addr}"),
    }
    println!(
        "address_book {}",
        aira_peer::AddressBook::path(root).display()
    );
    Ok(ExitCode::SUCCESS)
}

pub(super) async fn list(root: &Path) -> Result<ExitCode> {
    let book = aira_peer::AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    if book.peers.is_empty() {
        println!("(empty address book)");
    } else {
        for p in &book.peers {
            match &p.via {
                Some(v) => println!("{}\t{}\tvia {}", p.identity_id, p.addr, v),
                None => println!("{}\t{}", p.identity_id, p.addr),
            }
        }
    }
    println!(
        "address_book {}",
        aira_peer::AddressBook::path(root).display()
    );
    Ok(ExitCode::SUCCESS)
}

pub(super) async fn discovery(root: &Path) -> Result<ExitCode> {
    let store = aira_peer::PeerDiscoveryStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    if store.peers.is_empty() {
        println!("(empty discovery)");
    } else {
        for e in &store.peers {
            let addr = e.addr.as_deref().unwrap_or("-");
            let from = e.learned_from.as_deref().unwrap_or("-");
            let src = match e.source {
                aira_peer::DiscoverySource::Direct => "direct",
                aira_peer::DiscoverySource::Gossip => "gossip",
            };
            println!(
                "{}\t{}\t{}\t{}\t{}",
                e.identity_id, addr, e.last_seen, from, src
            );
        }
    }
    println!(
        "discovery {}",
        aira_peer::PeerDiscoveryStore::path(root).display()
    );
    Ok(ExitCode::SUCCESS)
}
