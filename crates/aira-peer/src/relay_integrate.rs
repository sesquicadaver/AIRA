//! Relay product path: dial order + signed ads + dual reservation (QUEUE #241 / Phase N).
//!
//! Transport fallback only — not discovery authority. End-to-end Noise remains peer↔peer;
//! courier hubs never verify payloads on behalf of endpoints. Full NAT/relay Noise
//! integration smoke stays `#246`.

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{
    descriptor_signing_message, AiraRef, Keyring, Signature, Timestamp, TrustStore,
    LOCAL_TEST_KEY_REF,
};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::PeerError;
use crate::presence_promote::trust_policy_allows;
use crate::prime_port::validate_aira_bind;
use crate::reachability_state::RelayRouteRecord;
use crate::stun::StunReflexiveRecord;

/// Schema for signed relay advertisements.
pub const RELAY_ADVERTISEMENT_SCHEMA: &str = "aira:schema:peer:relay-advertisement:0.1";

/// Durable cache of validated relay ads (`peers/relay_ads.json`).
pub const RELAY_ADS_STATE_SCHEMA: &str = "aira:peer:relay-ads:0.1";

/// Public mesh network id (shared with Presence).
pub const RELAY_AD_NETWORK_ID: &str = crate::presence::PUBLIC_NETWORK_ID;

/// SHOULD keep this many independent relay reservations when RELAY_ONLY (TZ §20).
pub const RELAY_RESERVATION_TARGET: usize = 2;

/// One step in the product dial order (TZ §19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialPathStep {
    /// AddressBook / Presence direct `tcp-peer` endpoint.
    Direct { addr: String },
    /// Observed NAT/reflexive endpoint (STUN hint — not reachability proof).
    NatObserved { addr: String },
    /// Courier via trusted relay (Noise still end-to-end at session layer).
    Relay {
        via_identity: String,
        relay_endpoint: String,
        reservation_id: Option<String>,
    },
}

impl DialPathStep {
    /// Stable kind label for tests / CLI.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Direct { .. } => "direct",
            Self::NatObserved { .. } => "nat",
            Self::Relay { .. } => "relay",
        }
    }
}

/// Inputs for planning a peer dial path (no live sockets).
#[derive(Debug, Clone, Default)]
pub struct DialPathInput {
    /// Preferred direct dial address (`host:port`, prime).
    pub direct_addr: Option<String>,
    /// Optional STUN/reflexive or observed NAT endpoint (`host:port`, prime).
    pub nat_observed_addr: Option<String>,
    /// Relay candidates (identity + endpoint + optional reservation).
    pub relays: Vec<RelayRouteRecord>,
}

/// Build ordered dial attempts: direct → NAT → relay(s). Dedupes identical addrs.
pub fn plan_dial_path(input: DialPathInput) -> Result<Vec<DialPathStep>, PeerError> {
    let mut steps = Vec::new();
    let mut seen_addrs = Vec::new();

    if let Some(addr) = input
        .direct_addr
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        validate_aira_bind(addr)?;
        seen_addrs.push(addr.to_string());
        steps.push(DialPathStep::Direct {
            addr: addr.to_string(),
        });
    }

    if let Some(addr) = input
        .nat_observed_addr
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        validate_aira_bind(addr)?;
        if !seen_addrs.iter().any(|a| a == addr) {
            seen_addrs.push(addr.to_string());
            steps.push(DialPathStep::NatObserved {
                addr: addr.to_string(),
            });
        }
    }

    for r in &input.relays {
        AiraRef::parse(&r.relay_identity_ref).map_err(|e| PeerError::Relay(e.to_string()))?;
        validate_aira_bind(&r.relay_endpoint)?;
        if r.relay_identity_ref.trim().is_empty() {
            return Err(PeerError::Relay("relay identity empty".into()));
        }
        steps.push(DialPathStep::Relay {
            via_identity: r.relay_identity_ref.clone(),
            relay_endpoint: r.relay_endpoint.clone(),
            reservation_id: r.reservation_id.clone(),
        });
    }

    if steps.is_empty() {
        return Err(PeerError::Relay(
            "dial path empty: need direct, NAT observed, or relay candidate".into(),
        ));
    }
    Ok(steps)
}

/// Load STUN reflexive addr from node root when present (optional NAT step).
pub fn nat_observed_from_root(root: impl AsRef<Path>) -> Option<String> {
    StunReflexiveRecord::load(root.as_ref())
        .ok()
        .map(|r| r.addr)
}

/// Signed courier capability advertisement (prime endpoint; trust-gated consumers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayAdvertisement {
    pub schema: String,
    pub network_id: String,
    pub relay_identity_ref: String,
    pub relay_public_key: String,
    pub relay_endpoint: String,
    pub created_at: String,
    pub expires_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_hint: Option<u32>,
    pub signature: Signature,
}

/// Draft inputs for an unsigned relay ad.
#[derive(Debug, Clone)]
pub struct RelayAdDraft {
    pub relay_identity_ref: String,
    pub relay_public_key: String,
    pub relay_endpoint: String,
    pub created_at: String,
    pub expires_at: String,
    pub capacity_hint: Option<u32>,
}

impl RelayAdvertisement {
    /// Build unsigned ad (placeholder signature).
    pub fn draft(input: RelayAdDraft) -> Result<Self, PeerError> {
        let id = AiraRef::parse(&input.relay_identity_ref)
            .map_err(|e| PeerError::Relay(e.to_string()))?;
        Ok(Self {
            schema: RELAY_ADVERTISEMENT_SCHEMA.into(),
            network_id: RELAY_AD_NETWORK_ID.into(),
            relay_identity_ref: input.relay_identity_ref,
            relay_public_key: input.relay_public_key,
            relay_endpoint: input.relay_endpoint,
            created_at: input.created_at,
            expires_at: input.expires_at,
            capacity_hint: input.capacity_hint,
            signature: Signature {
                algorithm: "ed25519".into(),
                key_ref: id,
                signature_value: String::new(),
            },
        })
    }

    /// Structural checks (no crypto).
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != RELAY_ADVERTISEMENT_SCHEMA {
            return Err(PeerError::Relay(format!(
                "relay ad schema mismatch: {}",
                self.schema
            )));
        }
        if self.network_id != RELAY_AD_NETWORK_ID {
            return Err(PeerError::Relay(format!(
                "relay ad network_id unsupported: {}",
                self.network_id
            )));
        }
        if self.relay_identity_ref == LOCAL_TEST_KEY_REF {
            return Err(PeerError::Untrusted(self.relay_identity_ref.clone()));
        }
        AiraRef::parse(&self.relay_identity_ref).map_err(|e| PeerError::Relay(e.to_string()))?;
        let pk = self.relay_public_key.trim();
        if pk.len() != 64 || !pk.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PeerError::Relay(
                "relay_public_key must be 64 hex chars".into(),
            ));
        }
        validate_aira_bind(&self.relay_endpoint)?;
        Timestamp::parse(&self.created_at).map_err(|e| PeerError::Relay(e.to_string()))?;
        Timestamp::parse(&self.expires_at).map_err(|e| PeerError::Relay(e.to_string()))?;
        if self.signature.key_ref.as_str() != self.relay_identity_ref {
            return Err(PeerError::IdentityMismatch);
        }
        Ok(())
    }

    /// Canonical-sign with a keyring holding `relay_identity_ref`.
    pub fn attach_canonical_signature(mut self, ring: &Keyring) -> Result<Self, PeerError> {
        self.validate_shape()?;
        let id = AiraRef::parse(&self.relay_identity_ref)
            .map_err(|e| PeerError::Relay(e.to_string()))?;
        let value = serde_json::to_value(&self)?;
        let msg = descriptor_signing_message(&value)?;
        self.signature = ring.sign(&id, &msg)?;
        if self.signature.key_ref.as_str() != self.relay_identity_ref {
            return Err(PeerError::IdentityMismatch);
        }
        Ok(self)
    }

    /// Sign using secrets from a relay node root.
    pub fn sign_for_node_root(self, root: impl AsRef<Path>) -> Result<Self, PeerError> {
        self.validate_shape()?;
        let (local_id, ring) = Keyring::load_node_identity(root.as_ref())?;
        if local_id.as_str() != self.relay_identity_ref {
            return Err(PeerError::Relay(
                "relay_identity_ref must match node identity".into(),
            ));
        }
        let expected_pk = ring
            .verifying_key(local_id.as_str())
            .ok_or_else(|| PeerError::Crypto("missing verifying key".into()))?;
        let pk_hex = hex::encode(expected_pk.as_bytes());
        if !self.relay_public_key.eq_ignore_ascii_case(&pk_hex) {
            return Err(PeerError::Relay(
                "relay_public_key does not match node verifying key".into(),
            ));
        }
        self.attach_canonical_signature(&ring)
    }

    /// Verify canonical signature against `relay_public_key` (no TrustStore upsert).
    pub fn verify_canonical_signature(&self) -> Result<(), PeerError> {
        self.validate_shape()?;
        if !aira_object::is_cryptographic_signature(&self.signature) {
            return Err(PeerError::InvalidSignature);
        }
        let mut store = TrustStore::default();
        store
            .upsert(&self.relay_identity_ref, self.relay_public_key.trim())
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

    /// True when `expires_at` is strictly after `now` (RFC3339).
    pub fn is_unexpired(&self, now: &str) -> Result<bool, PeerError> {
        let exp = parse_odt(&self.expires_at)?;
        let n = parse_odt(now)?;
        Ok(exp > n)
    }

    /// Convert to a reachability relay route row.
    pub fn to_route(&self, reservation_id: impl Into<String>) -> RelayRouteRecord {
        RelayRouteRecord {
            relay_identity_ref: self.relay_identity_ref.clone(),
            relay_endpoint: self.relay_endpoint.clone(),
            reservation_id: Some(reservation_id.into()),
        }
    }
}

/// Local cache of relay advertisements under `peers/relay_ads.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayAdStore {
    #[serde(default = "default_ads_schema")]
    pub schema: String,
    #[serde(default)]
    pub ads: Vec<RelayAdvertisement>,
}

fn default_ads_schema() -> String {
    RELAY_ADS_STATE_SCHEMA.into()
}

impl Default for RelayAdStore {
    fn default() -> Self {
        Self {
            schema: RELAY_ADS_STATE_SCHEMA.into(),
            ads: vec![],
        }
    }
}

impl RelayAdStore {
    /// Path: `<root>/peers/relay_ads.json`.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("relay_ads.json")
    }

    /// Load or empty.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self {
                schema: RELAY_ADS_STATE_SCHEMA.into(),
                ads: vec![],
            });
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::Io(e.to_string()))?;
        let store: Self =
            serde_json::from_str(&raw).map_err(|e| PeerError::Relay(e.to_string()))?;
        if store.schema != RELAY_ADS_STATE_SCHEMA {
            return Err(PeerError::Relay(format!(
                "relay ads schema mismatch: {}",
                store.schema
            )));
        }
        for ad in &store.ads {
            ad.validate_shape()?;
        }
        Ok(store)
    }

    /// Persist (creates `peers/`).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        if self.schema != RELAY_ADS_STATE_SCHEMA {
            return Err(PeerError::Relay(format!(
                "relay ads schema mismatch: {}",
                self.schema
            )));
        }
        for ad in &self.ads {
            ad.validate_shape()?;
        }
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::Io(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, format!("{json}\n")).map_err(|e| PeerError::Io(e.to_string()))?;
        Ok(())
    }

    /// Upsert by relay identity (replace prior ad for same id).
    pub fn upsert(&mut self, ad: RelayAdvertisement) -> Result<(), PeerError> {
        ad.validate_shape()?;
        if let Some(slot) = self
            .ads
            .iter_mut()
            .find(|a| a.relay_identity_ref == ad.relay_identity_ref)
        {
            *slot = ad;
        } else {
            self.ads.push(ad);
        }
        self.ads
            .sort_by(|a, b| a.relay_identity_ref.cmp(&b.relay_identity_ref));
        Ok(())
    }
}

/// Select up to [`RELAY_RESERVATION_TARGET`] trusted, valid, unexpired relay ads.
///
/// Distinct `relay_identity_ref` only. Does **not** upsert TrustStore.
pub fn select_relay_reservations(
    ads: &[RelayAdvertisement],
    trust: &TrustStore,
    now: &str,
    target: usize,
) -> Result<Vec<RelayRouteRecord>, PeerError> {
    let mut out = Vec::new();
    let mut seen = Vec::new();
    for (i, ad) in ads.iter().enumerate() {
        if out.len() >= target {
            break;
        }
        ad.verify_canonical_signature()?;
        if !ad.is_unexpired(now)? {
            continue;
        }
        if !trust_policy_allows(trust, &ad.relay_identity_ref) {
            continue;
        }
        if seen.iter().any(|id| id == &ad.relay_identity_ref) {
            continue;
        }
        seen.push(ad.relay_identity_ref.clone());
        out.push(ad.to_route(format!("res-{i}")));
    }
    Ok(out)
}

/// Convenience: plan dial path using book direct + optional STUN + selected relays.
pub fn plan_dial_path_with_relays(
    direct_addr: Option<String>,
    nat_observed_addr: Option<String>,
    relay_routes: Vec<RelayRouteRecord>,
) -> Result<Vec<DialPathStep>, PeerError> {
    plan_dial_path(DialPathInput {
        direct_addr,
        nat_observed_addr,
        relays: relay_routes,
    })
}

fn parse_odt(s: &str) -> Result<OffsetDateTime, PeerError> {
    OffsetDateTime::parse(s, &Rfc3339).map_err(|e| PeerError::Relay(format!("bad timestamp: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

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

    fn signed_ad(root: &Path, id: &AiraRef, pk: &str, endpoint: &str) -> RelayAdvertisement {
        RelayAdvertisement::draft(RelayAdDraft {
            relay_identity_ref: id.as_str().into(),
            relay_public_key: pk.into(),
            relay_endpoint: endpoint.into(),
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            capacity_hint: Some(8),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap()
    }

    #[test]
    fn dial_path_orders_direct_nat_then_relays() {
        let steps = plan_dial_path(DialPathInput {
            direct_addr: Some("127.0.0.1:49157".into()),
            nat_observed_addr: Some("127.0.0.1:49169".into()),
            relays: vec![RelayRouteRecord {
                relay_identity_ref: "aira:identity:hub".into(),
                relay_endpoint: "127.0.0.1:49171".into(),
                reservation_id: Some("r1".into()),
            }],
        })
        .unwrap();
        assert_eq!(
            steps.iter().map(DialPathStep::kind).collect::<Vec<_>>(),
            vec!["direct", "nat", "relay"]
        );
    }

    #[test]
    fn dial_path_dedupes_identical_direct_and_nat() {
        let steps = plan_dial_path(DialPathInput {
            direct_addr: Some("127.0.0.1:49157".into()),
            nat_observed_addr: Some("127.0.0.1:49157".into()),
            relays: vec![],
        })
        .unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].kind(), "direct");
    }

    #[test]
    fn dial_path_rejects_non_prime() {
        assert!(plan_dial_path(DialPathInput {
            direct_addr: Some("127.0.0.1:443".into()),
            nat_observed_addr: None,
            relays: vec![],
        })
        .is_err());
    }

    #[test]
    fn relay_ad_sign_verify_and_store() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "relay-a", [41u8; 32]);
        let ad = signed_ad(dir.path(), &id, &pk, "127.0.0.1:49157");
        ad.verify_canonical_signature().unwrap();
        let mut store = RelayAdStore::default();
        store.upsert(ad.clone()).unwrap();
        store.save(dir.path()).unwrap();
        let loaded = RelayAdStore::load(dir.path()).unwrap();
        assert_eq!(loaded.ads.len(), 1);
        loaded.ads[0].verify_canonical_signature().unwrap();
    }

    #[test]
    fn dual_reservation_picks_two_trusted_distinct() {
        let a = tempdir().unwrap();
        let b = tempdir().unwrap();
        let local = tempdir().unwrap();
        let (ida, pka) = write_node(a.path(), "relay-r1", [42u8; 32]);
        let (idb, pkb) = write_node(b.path(), "relay-r2", [43u8; 32]);
        let _ = write_node(local.path(), "client", [44u8; 32]);
        let ada = signed_ad(a.path(), &ida, &pka, "127.0.0.1:49157");
        let adb = signed_ad(b.path(), &idb, &pkb, "127.0.0.1:49169");
        let mut trust = TrustStore::load(local.path()).unwrap();
        trust.upsert(ida.as_str(), &pka).unwrap();
        trust.upsert(idb.as_str(), &pkb).unwrap();
        trust.save(local.path()).unwrap();
        let routes = select_relay_reservations(
            &[ada, adb],
            &trust,
            "2026-09-05T12:00:00Z",
            RELAY_RESERVATION_TARGET,
        )
        .unwrap();
        assert_eq!(routes.len(), 2);
        assert_ne!(routes[0].relay_identity_ref, routes[1].relay_identity_ref);

        let steps = plan_dial_path_with_relays(None, None, routes).unwrap();
        assert!(steps.iter().all(|s| s.kind() == "relay"));
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn untrusted_relay_ad_skipped_no_auto_trust() {
        let relay = tempdir().unwrap();
        let local = tempdir().unwrap();
        let (id, pk) = write_node(relay.path(), "stranger-relay", [45u8; 32]);
        let _ = write_node(local.path(), "client2", [46u8; 32]);
        let ad = signed_ad(relay.path(), &id, &pk, "127.0.0.1:49157");
        let trust = TrustStore::load(local.path()).unwrap();
        let before = trust.entries.len();
        let routes = select_relay_reservations(
            &[ad],
            &trust,
            "2026-09-05T12:00:00Z",
            RELAY_RESERVATION_TARGET,
        )
        .unwrap();
        assert!(routes.is_empty());
        let after = TrustStore::load(local.path()).unwrap();
        assert_eq!(before, after.entries.len());
        assert!(!after.entries.iter().any(|e| e.identity_id == id.as_str()));
    }

    #[test]
    fn expired_ad_skipped() {
        let relay = tempdir().unwrap();
        let local = tempdir().unwrap();
        let (id, pk) = write_node(relay.path(), "old-relay", [47u8; 32]);
        let _ = write_node(local.path(), "client3", [48u8; 32]);
        let ad = signed_ad(relay.path(), &id, &pk, "127.0.0.1:49157");
        let mut trust = TrustStore::load(local.path()).unwrap();
        trust.upsert(id.as_str(), &pk).unwrap();
        trust.save(local.path()).unwrap();
        let routes = select_relay_reservations(
            &[ad],
            &trust,
            "2026-09-20T12:00:00Z",
            RELAY_RESERVATION_TARGET,
        )
        .unwrap();
        assert!(routes.is_empty());
    }
}
