//! Presence refresh / republication (QUEUE #242 / Phase N).
//!
//! Regular TTL refresh bumps `sequence` and renews `created_at`/`expires_at`.
//! Endpoint change rebuilds Presence with only the new endpoints (old not
//! advertised as current). Stale records are filtered by `expires_at`.
//! Live notify of trusted peers over the peer protocol is orchestrated by
//! callers (CLI `#243`); this module lists AddressBook targets only.

use std::path::Path;

use aira_object::Timestamp;
use time::format_description::well_known::Rfc3339;
use time::Duration;
use time::OffsetDateTime;

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::presence::{
    NodePresenceRecord, PresenceDirectEndpoint, PresenceDraft, PresenceRelayEndpoint,
};
use crate::rendezvous_ops::{RENDEZVOUS_MAX_TTL_SECS, RENDEZVOUS_MIN_TTL_SECS};

/// Default refresh TTL (1 day) — within rendezvous policy bounds.
pub const PRESENCE_REFRESH_TTL_SECS_DEFAULT: u64 = 24 * 60 * 60;

fn parse_odt(s: &str) -> Result<OffsetDateTime, PeerError> {
    OffsetDateTime::parse(s.trim(), &Rfc3339)
        .map_err(|e| PeerError::Protocol(format!("bad timestamp {s}: {e}")))
}

fn format_odt(t: OffsetDateTime) -> String {
    t.format(&Rfc3339)
        .expect("OffsetDateTime formats as RFC3339")
}

/// True when `as_of` is at or after `expires_at` (stale / not active).
pub fn is_presence_expired(record: &NodePresenceRecord, as_of: &str) -> Result<bool, PeerError> {
    record.validate_shape()?;
    let _ = Timestamp::parse(as_of).map_err(|e| PeerError::Protocol(e.to_string()))?;
    Ok(parse_odt(as_of)? >= parse_odt(&record.expires_at)?)
}

/// Drop expired records so callers do not accumulate stale active endpoints.
pub fn retain_unexpired_presence(
    records: Vec<NodePresenceRecord>,
    as_of: &str,
) -> Result<Vec<NodePresenceRecord>, PeerError> {
    let mut out = Vec::with_capacity(records.len());
    for r in records {
        if !is_presence_expired(&r, as_of)? {
            out.push(r);
        }
    }
    Ok(out)
}

/// Compare advertised endpoints (direct + relay) for change detection.
pub fn presence_endpoints_equal(a: &NodePresenceRecord, b: &NodePresenceRecord) -> bool {
    a.direct_endpoints == b.direct_endpoints && a.relay_endpoints == b.relay_endpoints
}

fn validate_ttl_secs(ttl_secs: u64) -> Result<(), PeerError> {
    if ttl_secs < RENDEZVOUS_MIN_TTL_SECS {
        return Err(PeerError::Protocol(format!(
            "presence refresh TTL {ttl_secs}s below min {RENDEZVOUS_MIN_TTL_SECS}"
        )));
    }
    if ttl_secs > RENDEZVOUS_MAX_TTL_SECS {
        return Err(PeerError::Protocol(format!(
            "presence refresh TTL {ttl_secs}s above max {RENDEZVOUS_MAX_TTL_SECS}"
        )));
    }
    Ok(())
}

fn expires_after(now: &str, ttl_secs: u64) -> Result<String, PeerError> {
    validate_ttl_secs(ttl_secs)?;
    let base = parse_odt(now)?;
    let exp = base + Duration::seconds(ttl_secs as i64);
    Ok(format_odt(exp))
}

/// Build an unsigned refresh draft: `sequence++`, new created/expires, same endpoints.
pub fn refresh_presence_draft(
    previous: &NodePresenceRecord,
    now: &str,
    ttl_secs: u64,
) -> Result<PresenceDraft, PeerError> {
    previous.validate_shape()?;
    let _ = Timestamp::parse(now).map_err(|e| PeerError::Protocol(e.to_string()))?;
    let expires_at = expires_after(now, ttl_secs)?;
    Ok(PresenceDraft {
        identity_ref: previous.identity_ref.clone(),
        identity_public_key: previous.identity_public_key.clone(),
        sequence: previous
            .sequence
            .checked_add(1)
            .ok_or_else(|| PeerError::Protocol("presence sequence overflow".into()))?,
        created_at: now.to_string(),
        expires_at,
        direct_endpoints: previous.direct_endpoints.clone(),
        relay_endpoints: previous.relay_endpoints.clone(),
        capabilities_hash: previous.capabilities_hash.clone(),
    })
}

/// Endpoint-change republication: only the new endpoints are current.
///
/// Fails if the new endpoint set equals the previous (no-op change).
pub fn endpoint_change_presence_draft(
    previous: &NodePresenceRecord,
    now: &str,
    ttl_secs: u64,
    direct_endpoints: Vec<PresenceDirectEndpoint>,
    relay_endpoints: Vec<PresenceRelayEndpoint>,
) -> Result<PresenceDraft, PeerError> {
    previous.validate_shape()?;
    let _ = Timestamp::parse(now).map_err(|e| PeerError::Protocol(e.to_string()))?;
    let expires_at = expires_after(now, ttl_secs)?;
    let draft = PresenceDraft {
        identity_ref: previous.identity_ref.clone(),
        identity_public_key: previous.identity_public_key.clone(),
        sequence: previous
            .sequence
            .checked_add(1)
            .ok_or_else(|| PeerError::Protocol("presence sequence overflow".into()))?,
        created_at: now.to_string(),
        expires_at,
        direct_endpoints,
        relay_endpoints,
        capabilities_hash: previous.capabilities_hash.clone(),
    };
    // Validate shape via draft→record without signature.
    let probe = NodePresenceRecord::draft(draft.clone())?;
    probe.validate_shape()?;
    if presence_endpoints_equal(previous, &probe) {
        return Err(PeerError::Protocol(
            "endpoint change requires different direct/relay endpoints".into(),
        ));
    }
    Ok(draft)
}

/// Sign a refresh/endpoint-change draft for the local node root.
pub fn sign_presence_draft_for_node(
    root: impl AsRef<Path>,
    draft: PresenceDraft,
) -> Result<NodePresenceRecord, PeerError> {
    NodePresenceRecord::draft(draft)?.sign_for_node_root(root.as_ref())
}

/// Regular TTL refresh: draft + sign for node.
pub fn refresh_and_sign_presence(
    root: impl AsRef<Path>,
    previous: &NodePresenceRecord,
    now: &str,
    ttl_secs: u64,
) -> Result<NodePresenceRecord, PeerError> {
    let draft = refresh_presence_draft(previous, now, ttl_secs)?;
    sign_presence_draft_for_node(root, draft)
}

/// Endpoint change: draft + sign for node (old endpoints not carried).
pub fn endpoint_change_and_sign_presence(
    root: impl AsRef<Path>,
    previous: &NodePresenceRecord,
    now: &str,
    ttl_secs: u64,
    direct_endpoints: Vec<PresenceDirectEndpoint>,
    relay_endpoints: Vec<PresenceRelayEndpoint>,
) -> Result<NodePresenceRecord, PeerError> {
    let draft =
        endpoint_change_presence_draft(previous, now, ttl_secs, direct_endpoints, relay_endpoints)?;
    sign_presence_draft_for_node(root, draft)
}

/// AddressBook peer ids to notify after endpoint change (best-effort list).
///
/// Does not dial; CLI / runtime (`#243`+) send via existing peer protocol.
pub fn trusted_peers_to_notify(root: impl AsRef<Path>) -> Result<Vec<String>, PeerError> {
    let book = AddressBook::load(root.as_ref())?;
    let mut ids: Vec<String> = book.peers.iter().map(|p| p.identity_id.clone()).collect();
    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, Keyring};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::address_book::AddressBook;
    use crate::presence::{empty_capabilities_hash, PresenceReachability};

    fn write_node(root: &Path, name: &str, seed: [u8; 32]) -> (AiraRef, String) {
        let paths = NodePaths::new(root);
        fs::create_dir_all(paths.identity_dir()).unwrap();
        let sk = SigningKey::from_bytes(&seed);
        let pub_hex = hex::encode(sk.verifying_key().to_bytes());
        let id = format!("aira:identity:{name}");
        let id_ref = AiraRef::parse(&id).unwrap();
        fs::write(
            paths.identity_key(),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        let sig = sign_with_key(id_ref.clone(), &sk, id.as_bytes());
        let desc = serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": name,
            "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": sig
        });
        fs::write(
            paths.identity_json(),
            serde_json::to_string_pretty(&desc).unwrap(),
        )
        .unwrap();
        let _ = ensure_trust_defaults(root).unwrap();
        let (loaded, _): (AiraRef, Keyring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded, id_ref);
        (id_ref, pub_hex)
    }

    fn signed(
        root: &Path,
        id: &AiraRef,
        pk: &str,
        seq: u64,
        created: &str,
        expires: &str,
        port: u16,
    ) -> NodePresenceRecord {
        NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pk.into(),
            sequence: seq,
            created_at: created.into(),
            expires_at: expires.into(),
            direct_endpoints: vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: "127.0.0.1".into(),
                port,
                reachability_state: PresenceReachability::Unknown,
                observed_at: created.into(),
            }],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap()
    }

    #[test]
    fn refresh_bumps_sequence_and_ttl_window() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "ref-a", [51u8; 32]);
        let prev = signed(
            dir.path(),
            &id,
            &pk,
            1,
            "2026-09-05T12:00:00Z",
            "2026-09-06T12:00:00Z",
            49157,
        );
        let next = refresh_and_sign_presence(
            dir.path(),
            &prev,
            "2026-09-05T18:00:00Z",
            PRESENCE_REFRESH_TTL_SECS_DEFAULT,
        )
        .unwrap();
        assert_eq!(next.sequence, 2);
        assert_eq!(next.created_at, "2026-09-05T18:00:00Z");
        assert_eq!(next.expires_at, "2026-09-06T18:00:00Z");
        assert!(presence_endpoints_equal(&prev, &next));
        next.verify_canonical_signature().unwrap();
    }

    #[test]
    fn expire_stale_filters_active_set() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "ref-b", [52u8; 32]);
        let live = signed(
            dir.path(),
            &id,
            &pk,
            1,
            "2026-09-05T12:00:00Z",
            "2026-09-12T12:00:00Z",
            49157,
        );
        let stale = signed(
            dir.path(),
            &id,
            &pk,
            2,
            "2026-09-01T12:00:00Z",
            "2026-09-02T12:00:00Z",
            49169,
        );
        let kept =
            retain_unexpired_presence(vec![live.clone(), stale], "2026-09-05T18:00:00Z").unwrap();
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].sequence, 1);
        assert!(!is_presence_expired(&live, "2026-09-05T18:00:00Z").unwrap());
        assert!(is_presence_expired(&kept[0], "2026-09-12T12:00:00Z").unwrap());
    }

    #[test]
    fn endpoint_change_drops_old_and_increments() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "ref-c", [53u8; 32]);
        let prev = signed(
            dir.path(),
            &id,
            &pk,
            3,
            "2026-09-05T12:00:00Z",
            "2026-09-06T12:00:00Z",
            49157,
        );
        let next = endpoint_change_and_sign_presence(
            dir.path(),
            &prev,
            "2026-09-05T13:00:00Z",
            3600,
            vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: "127.0.0.1".into(),
                port: 49171,
                reachability_state: PresenceReachability::Unknown,
                observed_at: "2026-09-05T13:00:00Z".into(),
            }],
            vec![],
        )
        .unwrap();
        assert_eq!(next.sequence, 4);
        assert_eq!(next.direct_endpoints.len(), 1);
        assert_eq!(next.direct_endpoints[0].port, 49171);
        assert!(!presence_endpoints_equal(&prev, &next));
        // Same endpoints → reject
        assert!(endpoint_change_presence_draft(
            &prev,
            "2026-09-05T13:00:00Z",
            3600,
            prev.direct_endpoints.clone(),
            prev.relay_endpoints.clone(),
        )
        .is_err());
    }

    #[test]
    fn notify_list_from_address_book() {
        let dir = tempdir().unwrap();
        let _ = write_node(dir.path(), "ref-host", [54u8; 32]);
        assert!(trusted_peers_to_notify(dir.path()).unwrap().is_empty());
        let mut book = AddressBook::default();
        book.upsert("aira:identity:peer-z", "127.0.0.1:49157")
            .unwrap();
        book.upsert("aira:identity:peer-a", "127.0.0.1:49169")
            .unwrap();
        book.save(dir.path()).unwrap();
        assert_eq!(
            trusted_peers_to_notify(dir.path()).unwrap(),
            vec![
                "aira:identity:peer-a".to_string(),
                "aira:identity:peer-z".to_string()
            ]
        );
    }
}
