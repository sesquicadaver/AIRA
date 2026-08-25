//! Peer / DHT / STUN / discv commands (Analyze-81 / QUEUE #129 split).

mod book;
mod dht;
mod discv;
mod session;
mod stun;

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

use crate::cli::PeerCommands;

pub(crate) async fn run_peer(root: &Path, command: PeerCommands) -> Result<ExitCode> {
    match command {
        PeerCommands::Add { key_ref, addr, via } => book::add(root, key_ref, addr, via).await,
        PeerCommands::List => book::list(root).await,
        PeerCommands::Discovery => book::discovery(root).await,
        PeerCommands::Dht { command } => dht::run(root, command).await,
        PeerCommands::Stun { command } => stun::run(root, command).await,
        PeerCommands::Discv { command } => discv::run(root, command).await,
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
            session::listen(
                root,
                bind,
                once,
                recv,
                apply_trust,
                gossip,
                relay,
                relay_ttl_days,
                dht,
                apply_book,
            )
            .await
        }
        PeerCommands::RelayHold {
            key_ref,
            apply_trust,
        } => session::relay_hold(root, key_ref, apply_trust).await,
        PeerCommands::Dial { key_ref } => session::dial(root, key_ref).await,
        PeerCommands::Send { key_ref, text } => session::send(root, key_ref, text).await,
        PeerCommands::TrustSend {
            key_ref,
            op,
            subject,
            reason,
            new_id,
            pubkey_hex,
            until,
        } => {
            session::trust_send(
                root, key_ref, op, subject, reason, new_id, pubkey_hex, until,
            )
            .await
        }
        PeerCommands::NotifyRekey {
            key_ref,
            pubkey_hex,
            until,
        } => session::notify_rekey(root, key_ref, pubkey_hex, until).await,
    }
}
