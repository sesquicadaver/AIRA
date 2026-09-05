//! Node Presence Record (QUEUE #234 / Phase N).
//!
//! Canonical-signed discovery advertisement. Ledger publish uses `#235` trait / `#236+` adapters.
//! `DISCOVERED ≠ TRUSTED`: verifying a presence does not upsert TrustStore.

use aira_object::{
    descriptor_signing_message, utc_now_rfc3339, AiraRef, ContentHash, Keyring, Signature,
    Timestamp, TrustStore, LOCAL_TEST_KEY_REF,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::PeerError;
use crate::prime_port::{is_valid_aira_port, validate_aira_bind};

/// Schema `$id` / wire `schema` const.
pub const PRESENCE_SCHEMA: &str = "aira:schema:peer:presence-record:0.1";
/// Public mesh network id (Phase N invariant).
pub const PUBLIC_NETWORK_ID: &str = "aira:network:public:v1";

/// Direct endpoint reachability hint (full state machine lands in `#239`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PresenceReachability {
    Unknown,
    Direct,
    Nat,
    Relay,
    Offline,
}

/// AIRA-owned direct transport advertisement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceDirectEndpoint {
    pub transport: String,
    pub host: String,
    pub port: u16,
    pub reachability_state: PresenceReachability,
    pub observed_at: String,
}

/// Courier relay advertisement (optional).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceRelayEndpoint {
    pub relay_identity_ref: String,
    pub relay_endpoint: String,
    pub reservation_id: String,
    pub expires_at: String,
}

/// Signed node presence (canonical Ed25519 over descriptor without `signature`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePresenceRecord {
    pub schema: String,
    pub network_id: String,
    pub identity_ref: String,
    pub identity_public_key: String,
    pub sequence: u64,
    pub created_at: String,
    pub expires_at: String,
    pub direct_endpoints: Vec<PresenceDirectEndpoint>,
    pub relay_endpoints: Vec<PresenceRelayEndpoint>,
    pub capabilities_hash: String,
    pub signature: Signature,
}

/// Inputs for an unsigned presence draft (avoids a long `draft` argument list).
#[derive(Debug, Clone)]
pub struct PresenceDraft {
    pub identity_ref: String,
    pub identity_public_key: String,
    pub sequence: u64,
    pub created_at: String,
    pub expires_at: String,
    pub direct_endpoints: Vec<PresenceDirectEndpoint>,
    pub relay_endpoints: Vec<PresenceRelayEndpoint>,
    pub capabilities_hash: String,
}

impl NodePresenceRecord {
    /// Build an unsigned record (placeholder signature) for the local identity.
    pub fn draft(input: PresenceDraft) -> Result<Self, PeerError> {
        let id =
            AiraRef::parse(&input.identity_ref).map_err(|e| PeerError::Protocol(e.to_string()))?;
        Ok(Self {
            schema: PRESENCE_SCHEMA.into(),
            network_id: PUBLIC_NETWORK_ID.into(),
            identity_ref: input.identity_ref,
            identity_public_key: input.identity_public_key,
            sequence: input.sequence,
            created_at: input.created_at,
            expires_at: input.expires_at,
            direct_endpoints: input.direct_endpoints,
            relay_endpoints: input.relay_endpoints,
            capabilities_hash: input.capabilities_hash,
            signature: Signature {
                algorithm: "ed25519".into(),
                key_ref: id,
                signature_value: String::new(),
            },
        })
    }

    /// Fail-closed structural checks (no crypto).
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != PRESENCE_SCHEMA {
            return Err(PeerError::Protocol(format!(
                "presence schema mismatch: {}",
                self.schema
            )));
        }
        if self.network_id != PUBLIC_NETWORK_ID {
            return Err(PeerError::Protocol(format!(
                "presence network_id unsupported: {} (want {PUBLIC_NETWORK_ID})",
                self.network_id
            )));
        }
        if self.identity_ref == LOCAL_TEST_KEY_REF {
            return Err(PeerError::Untrusted(self.identity_ref.clone()));
        }
        AiraRef::parse(&self.identity_ref).map_err(|e| PeerError::Protocol(e.to_string()))?;
        let pk = self.identity_public_key.trim();
        if pk.len() != 64 || !pk.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PeerError::Protocol(
                "presence identity_public_key must be 64 hex chars".into(),
            ));
        }
        if self.sequence == 0 {
            return Err(PeerError::Protocol("presence sequence must be >= 1".into()));
        }
        Timestamp::parse(&self.created_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        Timestamp::parse(&self.expires_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        ContentHash::parse(&self.capabilities_hash)
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
        if self.signature.key_ref.as_str() != self.identity_ref {
            return Err(PeerError::IdentityMismatch);
        }
        for ep in &self.direct_endpoints {
            if ep.transport != "tcp-peer" && ep.transport != "udp-discv" {
                return Err(PeerError::Protocol(format!(
                    "presence direct transport unsupported: {}",
                    ep.transport
                )));
            }
            if ep.host.trim().is_empty() {
                return Err(PeerError::Protocol("presence direct host empty".into()));
            }
            if !is_valid_aira_port(ep.port) {
                return Err(PeerError::InvalidPort(format!(
                    "presence direct port {} not in P_AIRA",
                    ep.port
                )));
            }
            Timestamp::parse(&ep.observed_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        }
        for ep in &self.relay_endpoints {
            AiraRef::parse(&ep.relay_identity_ref)
                .map_err(|e| PeerError::Protocol(e.to_string()))?;
            validate_aira_bind(&ep.relay_endpoint)?;
            if ep.reservation_id.trim().is_empty() {
                return Err(PeerError::Protocol(
                    "presence relay reservation_id empty".into(),
                ));
            }
            Timestamp::parse(&ep.expires_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        }
        Ok(())
    }

    /// Canonical-sign with a signing keyring that holds `identity_ref`.
    pub fn attach_canonical_signature(mut self, ring: &Keyring) -> Result<Self, PeerError> {
        self.validate_shape()?;
        let id =
            AiraRef::parse(&self.identity_ref).map_err(|e| PeerError::Protocol(e.to_string()))?;
        let value = serde_json::to_value(&self)?;
        let msg = descriptor_signing_message(&value)?;
        self.signature = ring.sign(&id, &msg)?;
        if self.signature.key_ref.as_str() != self.identity_ref {
            return Err(PeerError::IdentityMismatch);
        }
        Ok(self)
    }

    /// Sign using secrets loaded from a node root.
    pub fn sign_for_node_root(self, root: impl AsRef<std::path::Path>) -> Result<Self, PeerError> {
        self.validate_shape()?;
        let (local_id, ring) = Keyring::load_node_identity(root.as_ref())?;
        if local_id.as_str() != self.identity_ref {
            return Err(PeerError::Protocol(
                "presence identity_ref must match node identity".into(),
            ));
        }
        let expected_pk = ring
            .verifying_key(local_id.as_str())
            .ok_or_else(|| PeerError::Crypto("missing verifying key".into()))?;
        let pk_hex = hex::encode(expected_pk.as_bytes());
        if !self.identity_public_key.eq_ignore_ascii_case(&pk_hex) {
            return Err(PeerError::Protocol(
                "presence identity_public_key does not match node verifying key".into(),
            ));
        }
        self.attach_canonical_signature(&ring)
    }

    /// Verify canonical signature against `identity_public_key` (no TrustStore upsert).
    pub fn verify_canonical_signature(&self) -> Result<(), PeerError> {
        self.validate_shape()?;
        if !aira_object::is_cryptographic_signature(&self.signature) {
            return Err(PeerError::InvalidSignature);
        }
        let mut store = TrustStore::default();
        store
            .upsert(&self.identity_ref, self.identity_public_key.trim())
            .map_err(|e| PeerError::Crypto(e.to_string()))?;
        let ring = store
            .to_keyring()
            .map_err(|e| PeerError::Crypto(e.to_string()))?;
        let value = serde_json::to_value(self)?;
        let msg = descriptor_signing_message(&value)?;
        ring.verify(&self.signature, &msg)
            .map_err(|_| PeerError::InvalidSignature)?;
        Ok(())
    }
}

/// Empty capabilities hash helper for tests / early nodes.
pub fn empty_capabilities_hash() -> String {
    ContentHash::sha256_bytes(b"{}").as_str().to_string()
}

/// RFC3339 now for drafts (falls back to fixed MVP clock via object clock).
pub fn presence_now() -> Result<String, PeerError> {
    utc_now_rfc3339().map_err(|e| PeerError::Protocol(e.to_string()))
}

/// JSON value view (tests / hashing).
pub fn presence_to_value(record: &NodePresenceRecord) -> Result<Value, PeerError> {
    Ok(serde_json::to_value(record)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    fn write_node(
        root: &std::path::Path,
        name: &str,
        seed: [u8; 32],
    ) -> (AiraRef, String, Keyring) {
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
        let (loaded_id, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded_id, id_ref);
        (id_ref, pub_hex, ring)
    }

    fn sample_direct() -> PresenceDirectEndpoint {
        PresenceDirectEndpoint {
            transport: "tcp-peer".into(),
            host: "127.0.0.1".into(),
            port: 49157,
            reachability_state: PresenceReachability::Unknown,
            observed_at: "2026-09-05T12:00:00Z".into(),
        }
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pub_hex, _ring) = write_node(root, "presence-alice", [31u8; 32]);
        let record = NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pub_hex,
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![sample_direct()],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap();
        record.verify_canonical_signature().unwrap();
        let v = presence_to_value(&record).unwrap();
        assert!(v.get("signature").is_some());
        let _ = presence_now().unwrap();
    }

    #[test]
    fn mutation_breaks_verify() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pub_hex, _) = write_node(root, "presence-mut", [33u8; 32]);
        let signed = NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pub_hex,
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![sample_direct()],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap();

        let mut port = signed.clone();
        port.direct_endpoints[0].port = 49171;
        assert!(port.verify_canonical_signature().is_err());

        let mut host = signed.clone();
        host.direct_endpoints[0].host = "10.0.0.1".into();
        assert!(host.verify_canonical_signature().is_err());

        let mut identity = signed.clone();
        identity.identity_ref = "aira:identity:other".into();
        identity.signature.key_ref = AiraRef::parse("aira:identity:other").unwrap();
        assert!(identity.verify_canonical_signature().is_err());

        let mut expiry = signed.clone();
        expiry.expires_at = "2026-09-20T12:00:00Z".into();
        assert!(expiry.verify_canonical_signature().is_err());

        let mut seq = signed.clone();
        seq.sequence = 2;
        assert!(seq.verify_canonical_signature().is_err());

        let mut relay = signed.clone();
        relay.relay_endpoints.push(PresenceRelayEndpoint {
            relay_identity_ref: "aira:identity:relay".into(),
            relay_endpoint: "127.0.0.1:49157".into(),
            reservation_id: "r1".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
        });
        assert!(relay.verify_canonical_signature().is_err());
    }

    #[test]
    fn rejects_non_prime_direct_port() {
        let mut rec = NodePresenceRecord::draft(PresenceDraft {
            identity_ref: "aira:identity:x".into(),
            identity_public_key: "aa".repeat(32),
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: "127.0.0.1".into(),
                port: 443,
                reachability_state: PresenceReachability::Unknown,
                observed_at: "2026-09-05T12:00:00Z".into(),
            }],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap();
        assert!(matches!(
            rec.validate_shape().unwrap_err(),
            PeerError::InvalidPort(_)
        ));
        rec.direct_endpoints[0].port = 49157;
        rec.validate_shape().unwrap();
    }

    #[test]
    fn rejects_wrong_network_id() {
        let mut rec = NodePresenceRecord::draft(PresenceDraft {
            identity_ref: "aira:identity:x".into(),
            identity_public_key: "ab".repeat(32),
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap();
        rec.network_id = "aira:network:other".into();
        assert!(rec.validate_shape().is_err());
    }
}
