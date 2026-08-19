//! Peer / DHT / STUN / discv commands (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};

use crate::cli::{PeerCommands, PeerDhtCommands, PeerDiscvCommands, PeerStunCommands};
use crate::support::{build_peer_ping, require_trusted};

pub(crate) async fn run_peer(root: &Path, command: PeerCommands) -> Result<ExitCode> {
    match command {
        PeerCommands::Add { key_ref, addr, via } => {
            require_trusted(root, &key_ref)?;
            addr.parse::<std::net::SocketAddr>()
                .with_context(|| format!("invalid addr {addr}"))?;
            if let Some(ref via_id) = via {
                require_trusted(root, via_id)?;
            }
            let mut book =
                aira_peer::AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
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
        PeerCommands::List => {
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
        PeerCommands::Discovery => {
            let store =
                aira_peer::PeerDiscoveryStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
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
        PeerCommands::Dht { command } => match command {
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
                let store =
                    aira_peer::PeerDhtStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
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
                let store =
                    aira_peer::PeerDhtStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
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
        },
        PeerCommands::Stun { command } => match command {
            PeerStunCommands::Query { stun_server } => {
                let server = stun_server.ok_or_else(|| {
                    anyhow::anyhow!(
                        "stun server required — pass --stun-server host:port or set AIRA_STUN_SERVER"
                    )
                })?;
                let rec = aira_peer::query_and_save_stun_reflexive(
                    root,
                    &server,
                    aira_peer::STUN_QUERY_TIMEOUT,
                )
                .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("stun reflexive {}", rec.addr);
                println!("stun_server {}", rec.stun_server);
                println!(
                    "stun_reflexive {}",
                    aira_peer::StunReflexiveRecord::path(root).display()
                );
                Ok(ExitCode::SUCCESS)
            }
        },
        PeerCommands::Discv { command } => match command {
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
                let advertised =
                    aira_peer::resolve_dht_announce_addr(root, addr.as_deref(), from_stun)
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
        },
        PeerCommands::Listen {
            bind,
            once,
            recv,
            apply_trust,
            gossip,
            relay,
            relay_ttl_days,
            dht,
            apply_book,
        } => {
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
                        if let Err(e) = aira_peer::serve_relay_peer(hub_c, peer, &root_c, ttl).await
                        {
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
                            let delta = aira_peer::parse_trust_delta(&env)
                                .map_err(|e| anyhow::anyhow!("{e}"))?;
                            aira_peer::apply_trust_delta(root, &env.issuer_identity, &delta)
                                .map_err(|e| anyhow::anyhow!("{e}"))?;
                            println!(
                                "applied trust-delta {:?}\tsubject {}",
                                delta.op, delta.subject_id
                            );
                            if gossip {
                                let results =
                                    aira_peer::gossip_forward_trust_delta(root, &env, &from)
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
                            let announce = aira_peer::parse_dht_announce(&env)
                                .map_err(|e| anyhow::anyhow!("{e}"))?;
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
                                                                Some(why) => println!(
                                                                    "gossip skipped ({why})"
                                                                ),
                                                                None => println!(
                                                                    "gossip skipped (duplicate)"
                                                                ),
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
        PeerCommands::RelayHold {
            key_ref,
            apply_trust,
        } => {
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
                                aira_peer::apply_trust_delta(root, &env.issuer_identity, &d)
                                    .map(|_| d)
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
        PeerCommands::Dial { key_ref } => {
            require_trusted(root, &key_ref)?;
            let peer = aira_peer::dial(root, &key_ref)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("dialed {}", peer.peer_id.as_str());
            println!("local {}", peer.local_id.as_str());
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::Send { key_ref, text } => {
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
        PeerCommands::TrustSend {
            key_ref,
            op,
            subject,
            reason,
            new_id,
            pubkey_hex,
            until,
        } => {
            require_trusted(root, &key_ref)?;
            let op = aira_peer::TrustDeltaOp::parse(&op).map_err(|e| anyhow::anyhow!("{e}"))?;
            let delta = match op {
                aira_peer::TrustDeltaOp::Revoke => aira_peer::TrustDelta::revoke(subject, reason),
                aira_peer::TrustDeltaOp::Unrevoke => aira_peer::TrustDelta::unrevoke(subject),
                aira_peer::TrustDeltaOp::Rotate => {
                    let new_id =
                        new_id.ok_or_else(|| anyhow::anyhow!("rotate requires --new-id"))?;
                    let pubkey_hex = pubkey_hex
                        .ok_or_else(|| anyhow::anyhow!("rotate requires --pubkey-hex"))?;
                    aira_peer::TrustDelta::rotate(subject, new_id, pubkey_hex, reason, until)
                }
                aira_peer::TrustDeltaOp::Rekey => {
                    let pubkey_hex =
                        pubkey_hex.ok_or_else(|| anyhow::anyhow!("rekey requires --pubkey-hex"))?;
                    aira_peer::TrustDelta::rekey(subject, pubkey_hex, reason, until)
                }
            };
            let env = aira_peer::make_trust_delta_envelope(root, &delta)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
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
        PeerCommands::NotifyRekey {
            key_ref,
            pubkey_hex,
            until,
        } => {
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
    }
}
