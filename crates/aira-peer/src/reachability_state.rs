//! Reachability state machine + `peers/reachability.json` (QUEUE #239 / Phase N).
//!
//! Persists local status (UNKNOWN…OFFLINE). AddressBook promotion remains `#240`.

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::Timestamp;
use serde::{Deserialize, Serialize};

use crate::error::PeerError;
use crate::presence::PresenceReachability;
use crate::prime_port::{is_valid_aira_port, parse_bind_port, validate_aira_bind};
use crate::reachability::ReachabilityResult;

/// Schema tag for local reachability state file.
pub const REACHABILITY_STATE_SCHEMA: &str = "aira:peer:reachability-state:0.1";

/// Local node reachability status (TZ §17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachabilityStatus {
    /// No external probe yet (first node / cold start).
    #[default]
    Unknown,
    /// Local bind succeeded; external inbound not proven.
    LocalOnly,
    /// At least one advertised prime endpoint verified by external probe.
    DirectReachable,
    /// Direct probe failed; relay route confirmed.
    RelayOnly,
    /// Outbound to ledger/peers works; no inbound direct or relay.
    OutboundOnly,
    /// No working global transport.
    Offline,
}

impl ReachabilityStatus {
    /// Map to Presence advertisement hint (coarse).
    pub fn to_presence_hint(self) -> PresenceReachability {
        match self {
            Self::Unknown | Self::LocalOnly => PresenceReachability::Unknown,
            Self::DirectReachable => PresenceReachability::Direct,
            Self::RelayOnly => PresenceReachability::Relay,
            Self::OutboundOnly => PresenceReachability::Nat,
            Self::Offline => PresenceReachability::Offline,
        }
    }

    /// True when Presence may advertise DIRECT (only after external proof).
    pub fn may_advertise_direct(self) -> bool {
        matches!(self, Self::DirectReachable)
    }
}

/// One relay route remembered for RELAY_ONLY.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayRouteRecord {
    pub relay_identity_ref: String,
    pub relay_endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation_id: Option<String>,
}

/// Local reachability snapshot under `peers/reachability.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachabilityLocalState {
    pub schema: String,
    pub status: ReachabilityStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_endpoint: Option<String>,
    #[serde(default)]
    pub relay_routes: Vec<RelayRouteRecord>,
    /// Last successful probe challenge_id (evidence pointer, not full result).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe_evidence: Option<String>,
}

impl Default for ReachabilityLocalState {
    fn default() -> Self {
        Self {
            schema: REACHABILITY_STATE_SCHEMA.into(),
            status: ReachabilityStatus::Unknown,
            checked_at: None,
            local_port: None,
            observed_endpoint: None,
            verified_endpoint: None,
            relay_routes: vec![],
            probe_evidence: None,
        }
    }
}

impl ReachabilityLocalState {
    /// Path: `<root>/peers/reachability.json`.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("reachability.json")
    }

    /// Load or default UNKNOWN.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::Io(e.to_string()))?;
        let st: Self =
            serde_json::from_str(&raw).map_err(|e| PeerError::Protocol(e.to_string()))?;
        st.validate()?;
        Ok(st)
    }

    /// Persist (creates `peers/`).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        self.validate()?;
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::Io(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, format!("{json}\n")).map_err(|e| PeerError::Io(e.to_string()))?;
        Ok(())
    }

    /// Structural checks.
    pub fn validate(&self) -> Result<(), PeerError> {
        if self.schema != REACHABILITY_STATE_SCHEMA {
            return Err(PeerError::Reachability(format!(
                "reachability state schema mismatch: {}",
                self.schema
            )));
        }
        if let Some(port) = self.local_port {
            if !is_valid_aira_port(port) {
                return Err(PeerError::InvalidPort(format!(
                    "reachability local_port {port} not in P_AIRA"
                )));
            }
        }
        if let Some(ep) = &self.observed_endpoint {
            validate_aira_bind(ep)?;
        }
        if let Some(ep) = &self.verified_endpoint {
            validate_aira_bind(ep)?;
        }
        if let Some(ts) = &self.checked_at {
            Timestamp::parse(ts).map_err(|e| PeerError::Protocol(e.to_string()))?;
        }
        for r in &self.relay_routes {
            aira_object::AiraRef::parse(&r.relay_identity_ref)
                .map_err(|e| PeerError::Protocol(e.to_string()))?;
            validate_aira_bind(&r.relay_endpoint)?;
        }
        if self.status == ReachabilityStatus::DirectReachable
            && !self.may_advertise_direct_consistent()
        {
            return Err(PeerError::Reachability(
                "DIRECT_REACHABLE requires verified_endpoint and probe_evidence".into(),
            ));
        }
        Ok(())
    }

    /// Coarse Presence advertisement hint from current status.
    pub fn to_presence_hint(&self) -> PresenceReachability {
        self.status.to_presence_hint()
    }

    /// True when Presence may advertise DIRECT (only after external proof).
    pub fn may_advertise_direct(&self) -> bool {
        self.status.may_advertise_direct()
    }

    fn may_advertise_direct_consistent(&self) -> bool {
        self.verified_endpoint.is_some() && self.probe_evidence.is_some()
    }

    /// Record local prime bind without external proof → LOCAL_ONLY (never DIRECT).
    pub fn mark_local_bind(
        &mut self,
        local_port: u16,
        checked_at: impl Into<String>,
    ) -> Result<(), PeerError> {
        if !is_valid_aira_port(local_port) {
            return Err(PeerError::InvalidPort(format!(
                "reachability local_port {local_port} not in P_AIRA"
            )));
        }
        let checked_at = checked_at.into();
        Timestamp::parse(&checked_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        self.local_port = Some(local_port);
        self.checked_at = Some(checked_at);
        if !matches!(
            self.status,
            ReachabilityStatus::DirectReachable | ReachabilityStatus::RelayOnly
        ) {
            self.status = ReachabilityStatus::LocalOnly;
        }
        self.validate()
    }

    /// Apply a verified successful peer-assisted probe → DIRECT_REACHABLE.
    pub fn apply_successful_probe(&mut self, result: &ReachabilityResult) -> Result<(), PeerError> {
        result.verify(None)?;
        if !result.attestation.success {
            return Err(PeerError::Reachability(
                "cannot apply unsuccessful probe as DIRECT".into(),
            ));
        }
        let endpoint = result.challenge.endpoint.clone();
        let port = parse_bind_port(&endpoint)?;
        self.status = ReachabilityStatus::DirectReachable;
        self.local_port = Some(port);
        self.observed_endpoint = Some(result.attestation.observed_endpoint.clone());
        self.verified_endpoint = Some(endpoint);
        self.checked_at = Some(result.attestation.probed_at.clone());
        self.probe_evidence = Some(result.challenge.challenge_id.clone());
        self.validate()
    }

    /// Direct inbound failed; remember relay routes → RELAY_ONLY (or OUTBOUND_ONLY if none).
    pub fn apply_direct_failed(
        &mut self,
        checked_at: impl Into<String>,
        relay_routes: Vec<RelayRouteRecord>,
        outbound_ok: bool,
    ) -> Result<(), PeerError> {
        let checked_at = checked_at.into();
        Timestamp::parse(&checked_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        for r in &relay_routes {
            aira_object::AiraRef::parse(&r.relay_identity_ref)
                .map_err(|e| PeerError::Protocol(e.to_string()))?;
            validate_aira_bind(&r.relay_endpoint)?;
        }
        self.checked_at = Some(checked_at);
        self.verified_endpoint = None;
        self.probe_evidence = None;
        self.relay_routes = relay_routes;
        self.status = if !self.relay_routes.is_empty() {
            ReachabilityStatus::RelayOnly
        } else if outbound_ok {
            ReachabilityStatus::OutboundOnly
        } else {
            ReachabilityStatus::Offline
        };
        // Never advertise DIRECT after failed direct probe.
        debug_assert!(!self.status.may_advertise_direct());
        self.validate()
    }

    /// Mark offline (no transport).
    pub fn mark_offline(&mut self, checked_at: impl Into<String>) -> Result<(), PeerError> {
        let checked_at = checked_at.into();
        Timestamp::parse(&checked_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        self.status = ReachabilityStatus::Offline;
        self.checked_at = Some(checked_at);
        self.verified_endpoint = None;
        self.probe_evidence = None;
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, Keyring};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::reachability::{
        ChallengeDraft, ReachabilityAttestation, ReachabilityChallenge, ReachabilityResult,
    };

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

    #[test]
    fn default_unknown_and_local_bind_never_direct() {
        let mut st = ReachabilityLocalState::default();
        assert_eq!(st.status, ReachabilityStatus::Unknown);
        assert!(!st.status.may_advertise_direct());
        st.mark_local_bind(49157, "2026-09-05T12:00:00Z").unwrap();
        assert_eq!(st.status, ReachabilityStatus::LocalOnly);
        assert!(!st.status.may_advertise_direct());
        assert_eq!(st.to_presence_hint(), PresenceReachability::Unknown);
    }

    #[test]
    fn successful_probe_sets_direct_and_persists() {
        let target = tempdir().unwrap();
        let probe = tempdir().unwrap();
        let root = tempdir().unwrap();
        let (tid, tpk) = write_node(target.path(), "st-tgt", [81u8; 32]);
        let _ = write_node(probe.path(), "st-prb", [82u8; 32]);
        let ch = ReachabilityChallenge::draft(ChallengeDraft {
            target_identity_ref: tid.as_str().into(),
            target_public_key: tpk,
            endpoint: "127.0.0.1:49157".into(),
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-05T13:00:00Z".into(),
        })
        .unwrap()
        .sign_for_node_root(target.path())
        .unwrap();
        let att = ReachabilityAttestation::issue_for_challenge(
            &ch,
            probe.path(),
            "127.0.0.1:49157",
            "2026-09-05T12:30:00Z",
            true,
        )
        .unwrap();
        let result = ReachabilityResult::new(ch, att);
        let mut st = ReachabilityLocalState::default();
        st.apply_successful_probe(&result).unwrap();
        assert_eq!(st.status, ReachabilityStatus::DirectReachable);
        assert!(st.status.may_advertise_direct());
        assert_eq!(st.to_presence_hint(), PresenceReachability::Direct);
        st.save(root.path()).unwrap();
        let loaded = ReachabilityLocalState::load(root.path()).unwrap();
        assert_eq!(loaded.status, ReachabilityStatus::DirectReachable);
        assert!(ReachabilityLocalState::path(root.path()).is_file());
    }

    #[test]
    fn direct_failed_with_relay_or_outbound_or_offline() {
        let mut st = ReachabilityLocalState::default();
        st.apply_direct_failed(
            "2026-09-05T12:00:00Z",
            vec![RelayRouteRecord {
                relay_identity_ref: "aira:identity:relay".into(),
                relay_endpoint: "127.0.0.1:49157".into(),
                reservation_id: Some("r1".into()),
            }],
            true,
        )
        .unwrap();
        assert_eq!(st.status, ReachabilityStatus::RelayOnly);
        assert!(!st.status.may_advertise_direct());

        st.apply_direct_failed("2026-09-05T12:01:00Z", vec![], true)
            .unwrap();
        assert_eq!(st.status, ReachabilityStatus::OutboundOnly);

        st.apply_direct_failed("2026-09-05T12:02:00Z", vec![], false)
            .unwrap();
        assert_eq!(st.status, ReachabilityStatus::Offline);
        st.mark_offline("2026-09-05T12:03:00Z").unwrap();
        assert_eq!(st.status, ReachabilityStatus::Offline);
    }

    #[test]
    fn rejects_non_prime_local_port() {
        let mut st = ReachabilityLocalState::default();
        assert!(st.mark_local_bind(443, "2026-09-05T12:00:00Z").is_err());
    }
}
