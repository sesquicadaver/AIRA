use std::path::Path;
use std::process::ExitCode;

use anyhow::{bail, Result};

use crate::support::{build_peer_ping, require_trusted};

#[allow(clippy::too_many_arguments)]
pub(super) async fn listen(
    root: &Path,
    bind: String,
    once: bool,
    recv: bool,
    apply_trust: bool,
    gossip: bool,
    relay: bool,
    relay_ttl_days: Option<u64>,
    dht: bool,
    apply_book: bool,
) -> Result<ExitCode> {
    if apply_trust && !recv && !relay {
        bail!("--apply-trust requires --recv (or use --relay)");
    }
    if dht && !recv && !relay {
        bail!("--dht requires --recv (or use with --relay separately)");
    }
    if apply_book && !dht {
        bail!("--apply-book requires --dht");
    }
    if relay_ttl_days.is_some() && !relay {
        bail!("--relay-ttl-days requires --relay");
    }
    if gossip && !apply_trust {
        bail!("--gossip requires --apply-trust");
    }
    if relay && gossip {
        bail!("--relay and --gossip are mutually exclusive in this slice");
    }
    if relay && dht {
        bail!("--relay and --dht are mutually exclusive in this slice");
    }
    if relay && recv {
        bail!("--relay implies hub mode; omit --recv");
    }
    if relay {
        // Fail closed before bind: load + prove registry is writable (Analyze-58).
        aira_peer::with_relay_hub_registry(root, relay_ttl_days, |_| Ok(()))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    let listener = aira_peer::listen(&bind)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let addr = listener.local_addr()?;
    println!("listening {addr}");
    if once {
        println!("mode once");
    } else {
        println!("mode daemon");
    }
    if relay {
        println!("relay hub enabled");
        if let Some(days) = relay_ttl_days {
            println!("relay_ttl_days {days}");
        } else {
            println!("relay_ttl_days off (offline history retained)");
        }
    }
    if recv {
        println!("recv enabled");
    }
    if apply_trust {
        println!("apply_trust enabled");
    }
    if gossip {
        println!("gossip enabled");
    }
    if dht {
        println!("dht apply enabled");
    }
    if apply_book {
        println!("apply_book enabled");
    }
    let root_owned = root.to_path_buf();
    if relay {
        let hub = aira_peer::RelayHub::new();
        loop {
            // Analyze-59: TCP accept only on the loop; handshake runs in-task.
            let stream = match aira_peer::accept_tcp(&listener).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("accept error: {e}");
                    if once {
                        return Err(anyhow::anyhow!("{e}"));
                    }
                    if matches!(e, aira_peer::PeerError::Io(_)) {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                    continue;
                }
            };
            let hub_c = hub.clone();
            let root_c = root_owned.clone();
            let ttl = relay_ttl_days;
            if once {
                let peer = aira_peer::complete_accept(stream, &root_c)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("relay registered {}", peer.peer_id.as_str());
                aira_peer::serve_relay_peer(hub_c, peer, &root_c, ttl)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                break;
            }
            tokio::spawn(async move {
                let peer = match aira_peer::complete_accept(stream, &root_c).await {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("accept handshake error: {e}");
                        return;
                    }
                };
                println!("relay registered {}", peer.peer_id.as_str());
                if let Err(e) = aira_peer::serve_relay_peer(hub_c, peer, &root_c, ttl).await {
                    eprintln!("relay session ended: {e}");
                }
            });
        }
        return Ok(ExitCode::SUCCESS);
    }
    loop {
        // Analyze-59: TCP accept only on the loop; handshake (+recv) off-path.
        let stream = match aira_peer::accept_tcp(&listener).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept error: {e}");
                if once {
                    return Err(anyhow::anyhow!("{e}"));
                }
                if matches!(e, aira_peer::PeerError::Io(_)) {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                continue;
            }
        };
        if once {
            let mut peer = aira_peer::complete_accept(stream, root)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("accepted {}", peer.peer_id.as_str());
            let _ = aira_peer::PeerDiscoveryStore::record_and_save(
                root,
                peer.peer_id.as_str(),
                None,
                None,
                aira_peer::DiscoverySource::Direct,
            );
            if recv {
                let env = if apply_trust {
                    peer.recv_envelope_allow_relayed_trust_delta().await
                } else {
                    peer.recv_envelope().await
                }
                .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!(
                    "received {}\t{}\t{}",
                    env.message_type,
                    env.message_id.as_str(),
                    env.issuer_identity.as_str()
                );
                if let Some(payload) = env.payload_ref.as_deref() {
                    println!("payload_ref {payload}");
                }
                if apply_trust && env.message_type == aira_peer::TRUST_DELTA_MESSAGE_TYPE {
                    let from = peer.peer_id.as_str().to_string();
                    let delta =
                        aira_peer::parse_trust_delta(&env).map_err(|e| anyhow::anyhow!("{e}"))?;
                    aira_peer::apply_trust_delta(root, &env.issuer_identity, &delta)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!(
                        "applied trust-delta {:?}\tsubject {}",
                        delta.op, delta.subject_id
                    );
                    if gossip {
                        let results = aira_peer::gossip_forward_trust_delta(root, &env, &from)
                            .await
                            .map_err(|e| anyhow::anyhow!("{e}"))?;
                        for r in results {
                            if r.skipped {
                                match r.error.as_deref() {
                                    Some(why) => println!("gossip skipped ({why})"),
                                    None => println!("gossip skipped (duplicate)"),
                                }
                            } else if r.ok {
                                println!("gossip -> {}", r.peer_id);
                            } else {
                                eprintln!(
                                    "gossip failed {}\t{}",
                                    r.peer_id,
                                    r.error.unwrap_or_default()
                                );
                            }
                        }
                    }
                }
                if dht && env.message_type == aira_peer::DHT_ANNOUNCE_MESSAGE_TYPE {
                    let announce =
                        aira_peer::parse_dht_announce(&env).map_err(|e| anyhow::anyhow!("{e}"))?;
                    aira_peer::apply_dht_announce_maybe_book(
                        root,
                        &env.issuer_identity,
                        &announce,
                        apply_book,
                    )
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!(
                        "applied dht-announce {}\t{}",
                        announce.identity_id, announce.addr
                    );
                    if apply_book {
                        println!("apply_book {}\t{}", announce.identity_id, announce.addr);
                    }
                }
            }
            break;
        }
        let root_bg = root_owned.clone();
        let do_recv = recv;
        let do_apply = apply_trust;
        let do_gossip = gossip;
        let do_dht = dht;
        let do_apply_book = apply_book;
        tokio::spawn(async move {
            let mut peer = match aira_peer::complete_accept(stream, &root_bg).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("accept handshake error: {e}");
                    return;
                }
            };
            println!("accepted {}", peer.peer_id.as_str());
            let _ = aira_peer::PeerDiscoveryStore::record_and_save(
                &root_bg,
                peer.peer_id.as_str(),
                None,
                None,
                aira_peer::DiscoverySource::Direct,
            );
            if !do_recv {
                return;
            }
            let from = peer.peer_id.as_str().to_string();
            let recv = if do_apply {
                peer.recv_envelope_allow_relayed_trust_delta().await
            } else {
                peer.recv_envelope().await
            };
            match recv {
                Ok(env) => {
                    println!(
                        "received {}\t{}\t{}",
                        env.message_type,
                        env.message_id.as_str(),
                        env.issuer_identity.as_str()
                    );
                    if let Some(payload) = env.payload_ref.as_deref() {
                        println!("payload_ref {payload}");
                    }
                    if do_apply && env.message_type == aira_peer::TRUST_DELTA_MESSAGE_TYPE {
                        match aira_peer::parse_trust_delta(&env).and_then(|d| {
                            aira_peer::apply_trust_delta(&root_bg, &env.issuer_identity, &d)
                                .map(|_| d)
                        }) {
                            Ok(delta) => {
                                println!(
                                    "applied trust-delta {:?}\tsubject {}",
                                    delta.op, delta.subject_id
                                );
                                if do_gossip {
                                    match aira_peer::gossip_forward_trust_delta(
                                        &root_bg, &env, &from,
                                    )
                                    .await
                                    {
                                        Ok(results) => {
                                            for r in results {
                                                if r.skipped {
                                                    match r.error.as_deref() {
                                                        Some(why) => {
                                                            println!("gossip skipped ({why})")
                                                        }
                                                        None => {
                                                            println!("gossip skipped (duplicate)")
                                                        }
                                                    }
                                                } else if r.ok {
                                                    println!("gossip -> {}", r.peer_id);
                                                } else {
                                                    eprintln!(
                                                        "gossip failed {}\t{}",
                                                        r.peer_id,
                                                        r.error.unwrap_or_default()
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("gossip error: {e}");
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!(
                                "apply_trust error from {}: {e}",
                                env.issuer_identity.as_str()
                            ),
                        }
                    }
                    if do_dht && env.message_type == aira_peer::DHT_ANNOUNCE_MESSAGE_TYPE {
                        match aira_peer::parse_dht_announce(&env).and_then(|a| {
                            aira_peer::apply_dht_announce_maybe_book(
                                &root_bg,
                                &env.issuer_identity,
                                &a,
                                do_apply_book,
                            )
                            .map(|_| a)
                        }) {
                            Ok(announce) => {
                                println!(
                                    "applied dht-announce {}\t{}",
                                    announce.identity_id, announce.addr
                                );
                                if do_apply_book {
                                    println!(
                                        "apply_book {}\t{}",
                                        announce.identity_id, announce.addr
                                    );
                                }
                            }
                            Err(e) => eprintln!("dht apply error: {e}"),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("recv error from {}: {e}", from);
                }
            }
        });
    }
    Ok(ExitCode::SUCCESS)
}

pub(super) async fn relay_hold(
    root: &Path,
    key_ref: String,
    apply_trust: bool,
) -> Result<ExitCode> {
    require_trusted(root, &key_ref)?;
    let mut peer = aira_peer::dial(root, &key_ref)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("relay-hold {}", peer.peer_id.as_str());
    if apply_trust {
        println!("apply_trust enabled");
    }
    loop {
        match peer.recv_envelope_allow_relayed().await {
            Ok(env) => {
                println!(
                    "received {}\t{}\t{}",
                    env.message_type,
                    env.message_id.as_str(),
                    env.issuer_identity.as_str()
                );
                if apply_trust && env.message_type == aira_peer::TRUST_DELTA_MESSAGE_TYPE {
                    match aira_peer::parse_trust_delta(&env).and_then(|d| {
                        aira_peer::apply_trust_delta(root, &env.issuer_identity, &d).map(|_| d)
                    }) {
                        Ok(delta) => println!(
                            "applied trust-delta {:?}\tsubject {}",
                            delta.op, delta.subject_id
                        ),
                        Err(e) => eprintln!("apply_trust error: {e}"),
                    }
                }
            }
            Err(e) => {
                eprintln!("relay-hold ended: {e}");
                return Err(anyhow::anyhow!("{e}"));
            }
        }
    }
}

pub(super) async fn dial(root: &Path, key_ref: String) -> Result<ExitCode> {
    require_trusted(root, &key_ref)?;
    let peer = aira_peer::dial(root, &key_ref)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("dialed {}", peer.peer_id.as_str());
    println!("local {}", peer.local_id.as_str());
    Ok(ExitCode::SUCCESS)
}

pub(super) async fn send(root: &Path, key_ref: String, text: String) -> Result<ExitCode> {
    require_trusted(root, &key_ref)?;
    let env = build_peer_ping(root, &text)?;
    aira_peer::send_envelope_to_peer(root, &key_ref, &env)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let via = aira_peer::AddressBook::load(root)
        .ok()
        .and_then(|b| b.via_of(&key_ref).map(|s| s.to_string()));
    match via {
        Some(v) => println!(
            "sent {}\t{}\t-> {} via {}",
            env.message_type,
            env.message_id.as_str(),
            key_ref,
            v
        ),
        None => println!(
            "sent {}\t{}\t-> {}",
            env.message_type,
            env.message_id.as_str(),
            key_ref
        ),
    }
    Ok(ExitCode::SUCCESS)
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn trust_send(
    root: &Path,
    key_ref: String,
    op: String,
    subject: String,
    reason: Option<String>,
    new_id: Option<String>,
    pubkey_hex: Option<String>,
    until: Option<String>,
) -> Result<ExitCode> {
    require_trusted(root, &key_ref)?;
    let op = aira_peer::TrustDeltaOp::parse(&op).map_err(|e| anyhow::anyhow!("{e}"))?;
    let delta = match op {
        aira_peer::TrustDeltaOp::Revoke => aira_peer::TrustDelta::revoke(subject, reason),
        aira_peer::TrustDeltaOp::Unrevoke => aira_peer::TrustDelta::unrevoke(subject),
        aira_peer::TrustDeltaOp::Rotate => {
            let new_id = new_id.ok_or_else(|| anyhow::anyhow!("rotate requires --new-id"))?;
            let pubkey_hex =
                pubkey_hex.ok_or_else(|| anyhow::anyhow!("rotate requires --pubkey-hex"))?;
            aira_peer::TrustDelta::rotate(subject, new_id, pubkey_hex, reason, until)
        }
        aira_peer::TrustDeltaOp::Rekey => {
            let pubkey_hex =
                pubkey_hex.ok_or_else(|| anyhow::anyhow!("rekey requires --pubkey-hex"))?;
            aira_peer::TrustDelta::rekey(subject, pubkey_hex, reason, until)
        }
    };
    let env =
        aira_peer::make_trust_delta_envelope(root, &delta).map_err(|e| anyhow::anyhow!("{e}"))?;
    aira_peer::send_envelope_to_peer(root, &key_ref, &env)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let via = aira_peer::AddressBook::load(root)
        .ok()
        .and_then(|b| b.via_of(&key_ref).map(|s| s.to_string()));
    match via {
        Some(v) => println!(
            "sent {}\t{:?}\t{}\t-> {} via {}",
            env.message_type, delta.op, delta.subject_id, key_ref, v
        ),
        None => println!(
            "sent {}\t{:?}\t{}\t-> {}",
            env.message_type, delta.op, delta.subject_id, key_ref
        ),
    }
    Ok(ExitCode::SUCCESS)
}

pub(super) async fn notify_rekey(
    root: &Path,
    key_ref: Option<String>,
    pubkey_hex: String,
    until: Option<String>,
) -> Result<ExitCode> {
    if let Some(key_ref) = key_ref {
        require_trusted(root, &key_ref)?;
        aira_peer::notify_peer_of_rekey(root, &key_ref, &pubkey_hex, until.as_deref())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("notified {key_ref}");
    } else {
        let results = aira_peer::notify_peers_of_rekey(root, &pubkey_hex, until.as_deref())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if results.is_empty() {
            println!("notify_rekey (empty address book)");
        } else {
            for r in results {
                if r.ok {
                    println!("notified {}", r.peer_id);
                } else {
                    eprintln!(
                        "notify failed {}\t{}",
                        r.peer_id,
                        r.error.unwrap_or_default()
                    );
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
