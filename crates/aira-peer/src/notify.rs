//! Notify address-book peers of a local pubkey rekey (Analyze-38).

use std::path::Path;

use aira_object::Keyring;

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::session::dial;
use crate::trust_delta::{make_trust_delta_envelope, TrustDelta};

/// Outcome of one best-effort peer notify attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyPeerResult {
    pub peer_id: String,
    pub ok: bool,
    pub error: Option<String>,
}

/// Build a rekey delta for `new_pubkey_hex` under the local identity id.
///
/// Must be signed/sent **before** `rotate_node_signing_secret` so hello still verifies
/// under the peer's existing trust entry.
pub fn upcoming_rekey_delta(
    root: impl AsRef<Path>,
    new_pubkey_hex: &str,
    grace_until: Option<&str>,
) -> Result<TrustDelta, PeerError> {
    let (local_id, _) = Keyring::load_node_identity(root.as_ref())?;
    if new_pubkey_hex.trim().len() != 64 {
        return Err(PeerError::Protocol(
            "new_pubkey_hex must be 64 hex chars".into(),
        ));
    }
    Ok(TrustDelta::rekey(
        local_id.as_str(),
        new_pubkey_hex.trim(),
        Some("node signing secret rotated".into()),
        grace_until.map(|s| s.to_string()),
    ))
}

/// Dial each address-book peer and send a `rekey` trust-delta for `new_pubkey_hex` (best-effort).
///
/// Call **before** rotating the local signing secret so Noise/hello still authenticate
/// under the peers' current trust entry.
pub async fn notify_peers_of_rekey(
    root: impl AsRef<Path>,
    new_pubkey_hex: &str,
    grace_until: Option<&str>,
) -> Result<Vec<NotifyPeerResult>, PeerError> {
    let root = root.as_ref();
    let book = AddressBook::load(root)?;
    if book.peers.is_empty() {
        return Ok(vec![]);
    }
    let delta = upcoming_rekey_delta(root, new_pubkey_hex, grace_until)?;
    let env = make_trust_delta_envelope(root, &delta)?;
    let mut out = Vec::with_capacity(book.peers.len());
    for peer in &book.peers {
        let peer_id = peer.identity_id.clone();
        match dial(root, &peer_id).await {
            Ok(mut session) => match session.send_envelope(&env).await {
                Ok(()) => out.push(NotifyPeerResult {
                    peer_id,
                    ok: true,
                    error: None,
                }),
                Err(e) => out.push(NotifyPeerResult {
                    peer_id,
                    ok: false,
                    error: Some(e.to_string()),
                }),
            },
            Err(e) => out.push(NotifyPeerResult {
                peer_id,
                ok: false,
                error: Some(e.to_string()),
            }),
        }
    }
    Ok(out)
}

/// Notify a single address-book peer of an upcoming rekey (`new_pubkey_hex`).
pub async fn notify_peer_of_rekey(
    root: impl AsRef<Path>,
    peer_identity_id: &str,
    new_pubkey_hex: &str,
    grace_until: Option<&str>,
) -> Result<(), PeerError> {
    let root = root.as_ref();
    let delta = upcoming_rekey_delta(root, new_pubkey_hex, grace_until)?;
    let env = make_trust_delta_envelope(root, &delta)?;
    let mut session = dial(root, peer_identity_id).await?;
    session.send_envelope(&env).await
}
