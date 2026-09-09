//! Reachability state machine + `peers/reachability.json` (QUEUE #239 / Phase N).
//!
//! Persists local status (UNKNOWN…OFFLINE). AddressBook promotion remains `#240`.
//! `#279`: local bind / external DIRECT / relay keep independent observation times —
//! `mark_local_bind` must not refresh external/relay freshness.
//! `#281`: DIRECT apply is root-bound, durable-replayed, and apply-time fresh.
//! `#296`: challenge `target_public_key` must match authoritative root keyring (same-ID / foreign-key reject).

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{Keyring, Timestamp};
use serde::{Deserialize, Serialize};

use crate::error::PeerError;
use crate::presence::PresenceReachability;
use crate::prime_port::{is_valid_aira_port, parse_bind_port, validate_aira_bind};
use crate::reachability::{
    check_apply_time_freshness, load_reachability_replay, save_reachability_replay,
    ReachabilityLocalEvidence, ReachabilityResult,
};

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
    /// Status-facing observation time (legacy + CLI). Prefer [`Self::status_observation_at`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
    /// Last local bind observation (`#279`) — never proves external DIRECT.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_checked_at: Option<String>,
    /// Last successful external DIRECT probe (`#279`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_checked_at: Option<String>,
    /// Last relay-confirmed observation (`#279`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay_checked_at: Option<String>,
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
            local_checked_at: None,
            external_checked_at: None,
            relay_checked_at: None,
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

    /// Observation time that governs status freshness for the current banner (`#279`).
    ///
    /// Falls back to legacy `checked_at` for pre-`#279` files.
    pub fn status_observation_at(&self) -> Option<&str> {
        let primary = match self.status {
            ReachabilityStatus::DirectReachable => self.external_checked_at.as_deref(),
            ReachabilityStatus::RelayOnly => self.relay_checked_at.as_deref(),
            ReachabilityStatus::LocalOnly => self.local_checked_at.as_deref(),
            ReachabilityStatus::Unknown
            | ReachabilityStatus::OutboundOnly
            | ReachabilityStatus::Offline => None,
        };
        primary.or(self.checked_at.as_deref())
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
        for ts in [
            self.checked_at.as_deref(),
            self.local_checked_at.as_deref(),
            self.external_checked_at.as_deref(),
            self.relay_checked_at.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
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
    ///
    /// When status already is DIRECT/RELAY, only updates `local_port` /
    /// `local_checked_at` — does **not** refresh external/relay freshness (`#279`).
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
        self.local_checked_at = Some(checked_at.clone());
        if matches!(
            self.status,
            ReachabilityStatus::DirectReachable | ReachabilityStatus::RelayOnly
        ) {
            // Preserve external/relay observation clocks and legacy status checked_at.
        } else {
            self.status = ReachabilityStatus::LocalOnly;
            self.checked_at = Some(checked_at);
        }
        self.validate()
    }

    /// Apply a verified successful peer-assisted probe → DIRECT_REACHABLE (`#281` / `#296`).
    ///
    /// Admission requires:
    /// - `result.challenge.target_identity_ref` == current root node identity (`#281`)
    /// - `result.challenge.target_public_key` == authoritative root-scoped local
    ///   verifying key for that identity (`#296`) — same-ID / foreign-key packages
    ///   are rejected even when signatures are self-consistent under the embedded key
    /// - apply-time freshness (challenge not expired at `applied_at`;
    ///   `|applied_at − probed_at|` and evidence `created_at` within skew —
    ///   not only the signed `probed_at` window)
    /// - durable challenge-id replay under `peers/reachability_replay.json`
    /// - signed inbound [`ReachabilityLocalEvidence`] bound to the challenge
    ///
    /// A bare CLI transcript string is not accepted.
    pub fn apply_successful_probe(
        &mut self,
        root: impl AsRef<Path>,
        result: &ReachabilityResult,
        evidence: &ReachabilityLocalEvidence,
        applied_at: impl Into<String>,
    ) -> Result<(), PeerError> {
        let root = root.as_ref();
        let applied_at = applied_at.into();
        let (local_id, ring) = Keyring::load_node_identity(root)?;
        if local_id.as_str() != result.challenge.target_identity_ref {
            return Err(PeerError::Reachability(
                "reachability evidence apply requires challenge target = current root identity (#281)"
                    .into(),
            ));
        }
        let expected_pk = ring.verifying_key(local_id.as_str()).ok_or_else(|| {
            PeerError::Crypto("missing verifying key for root identity (#296)".into())
        })?;
        let expected_pk_hex = hex::encode(expected_pk.as_bytes());
        if !result
            .challenge
            .target_public_key
            .eq_ignore_ascii_case(&expected_pk_hex)
        {
            return Err(PeerError::Reachability(
                "reachability evidence apply requires challenge target_public_key = current root key (#296)"
                    .into(),
            ));
        }
        check_apply_time_freshness(
            &result.challenge,
            &result.attestation,
            evidence,
            &applied_at,
        )?;
        let mut replay = load_reachability_replay(root)?;
        result.verify_with_local_evidence(evidence, Some(&mut replay))?;
        if !result.attestation.success {
            return Err(PeerError::Reachability(
                "cannot apply unsuccessful probe as DIRECT".into(),
            ));
        }
        save_reachability_replay(root, &replay)?;
        let endpoint = result.challenge.endpoint.clone();
        let port = parse_bind_port(&endpoint)?;
        let probed_at = result.attestation.probed_at.clone();
        self.status = ReachabilityStatus::DirectReachable;
        self.local_port = Some(port);
        self.observed_endpoint = Some(result.attestation.observed_endpoint.clone());
        self.verified_endpoint = Some(endpoint);
        self.external_checked_at = Some(probed_at.clone());
        self.checked_at = Some(probed_at);
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
        self.checked_at = Some(checked_at.clone());
        self.verified_endpoint = None;
        self.probe_evidence = None;
        self.relay_routes = relay_routes;
        self.status = if !self.relay_routes.is_empty() {
            self.relay_checked_at = Some(checked_at);
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
        assert_eq!(st.local_checked_at.as_deref(), Some("2026-09-05T12:00:00Z"));
        assert_eq!(st.status_observation_at(), Some("2026-09-05T12:00:00Z"));
    }

    #[test]
    fn local_bind_does_not_refresh_external_direct_freshness() {
        let mut st = ReachabilityLocalState {
            status: ReachabilityStatus::DirectReachable,
            verified_endpoint: Some("127.0.0.1:49157".into()),
            probe_evidence: Some("ch-1".into()),
            external_checked_at: Some("2026-09-05T10:00:00Z".into()),
            checked_at: Some("2026-09-05T10:00:00Z".into()),
            local_port: Some(49157),
            ..Default::default()
        };
        st.mark_local_bind(49157, "2026-09-05T12:00:00Z").unwrap();
        assert_eq!(st.status, ReachabilityStatus::DirectReachable);
        assert_eq!(
            st.external_checked_at.as_deref(),
            Some("2026-09-05T10:00:00Z")
        );
        assert_eq!(st.checked_at.as_deref(), Some("2026-09-05T10:00:00Z"));
        assert_eq!(st.local_checked_at.as_deref(), Some("2026-09-05T12:00:00Z"));
        assert_eq!(st.status_observation_at(), Some("2026-09-05T10:00:00Z"));
    }

    #[test]
    fn local_bind_does_not_refresh_relay_freshness() {
        let mut st = ReachabilityLocalState::default();
        st.apply_direct_failed(
            "2026-09-05T10:00:00Z",
            vec![RelayRouteRecord {
                relay_identity_ref: "aira:identity:relay".into(),
                relay_endpoint: "127.0.0.1:49157".into(),
                reservation_id: Some("r1".into()),
            }],
            true,
        )
        .unwrap();
        assert_eq!(st.status, ReachabilityStatus::RelayOnly);
        st.mark_local_bind(49157, "2026-09-05T12:00:00Z").unwrap();
        assert_eq!(st.status, ReachabilityStatus::RelayOnly);
        assert_eq!(st.relay_checked_at.as_deref(), Some("2026-09-05T10:00:00Z"));
        assert_eq!(st.checked_at.as_deref(), Some("2026-09-05T10:00:00Z"));
        assert_eq!(st.local_checked_at.as_deref(), Some("2026-09-05T12:00:00Z"));
        assert_eq!(st.status_observation_at(), Some("2026-09-05T10:00:00Z"));
    }

    #[tokio::test]
    async fn successful_probe_sets_direct_and_persists() {
        use crate::{accept, admit_peer_trust, dial, listen_available_loopback, AddressBook};

        let target = tempdir().unwrap();
        let probe = tempdir().unwrap();
        let (tid, tpk) = write_node(target.path(), "st-tgt", [81u8; 32]);
        let (pid, ppk) = write_node(probe.path(), "st-prb", [82u8; 32]);
        admit_peer_trust(target.path(), pid.as_str(), &ppk).unwrap();
        admit_peer_trust(probe.path(), tid.as_str(), &tpk).unwrap();

        let (listener, addr) = listen_available_loopback().await.unwrap();
        let endpoint = format!("127.0.0.1:{}", addr.port());
        let mut book = AddressBook::default();
        book.upsert(tid.as_str(), &endpoint).unwrap();
        book.save(probe.path()).unwrap();

        let root_t = target.path().to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_t).await });
        let probe_session = dial(probe.path(), tid.as_str()).await.unwrap();
        let target_session = accept_task.await.unwrap().unwrap();

        let ch = ReachabilityChallenge::draft(ChallengeDraft {
            target_identity_ref: tid.as_str().into(),
            target_public_key: tpk,
            endpoint: endpoint.clone(),
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-05T13:00:00Z".into(),
        })
        .unwrap()
        .sign_for_node_root(target.path())
        .unwrap();
        let evidence = target_session
            .export_reachability_evidence(&ch, "2026-09-05T12:30:00Z")
            .unwrap();
        let att = ReachabilityAttestation::issue_for_authenticated_session(
            &ch,
            &probe_session,
            "2026-09-05T12:30:00Z",
        )
        .unwrap();
        let result = ReachabilityResult::new(ch, att);
        let mut st = ReachabilityLocalState::default();
        st.apply_successful_probe(target.path(), &result, &evidence, "2026-09-05T12:30:00Z")
            .unwrap();
        assert_eq!(st.status, ReachabilityStatus::DirectReachable);
        assert!(st.status.may_advertise_direct());
        assert_eq!(st.to_presence_hint(), PresenceReachability::Direct);
        assert_eq!(
            st.external_checked_at.as_deref(),
            Some("2026-09-05T12:30:00Z")
        );
        st.save(target.path()).unwrap();
        let loaded = ReachabilityLocalState::load(target.path()).unwrap();
        assert_eq!(loaded.status, ReachabilityStatus::DirectReachable);
        assert!(ReachabilityLocalState::path(target.path()).is_file());
        assert!(crate::reachability::reachability_replay_path(target.path()).is_file());

        // forged / transcript-alone style evidence cannot set DIRECT
        let mut forged = evidence.clone();
        forged.session_transcript_hex = "sha256:00".into();
        assert!(st
            .apply_successful_probe(target.path(), &result, &forged, "2026-09-05T12:30:00Z")
            .is_err());
    }

    #[tokio::test]
    async fn apply_rejects_wrong_root_replay_and_stale_apply_time() {
        use crate::{accept, admit_peer_trust, dial, listen_available_loopback, AddressBook};

        let target = tempdir().unwrap();
        let probe = tempdir().unwrap();
        let other = tempdir().unwrap();
        let (tid, tpk) = write_node(target.path(), "adm-tgt", [85u8; 32]);
        let (pid, ppk) = write_node(probe.path(), "adm-prb", [86u8; 32]);
        let (_oid, _) = write_node(other.path(), "adm-oth", [87u8; 32]);
        admit_peer_trust(target.path(), pid.as_str(), &ppk).unwrap();
        admit_peer_trust(probe.path(), tid.as_str(), &tpk).unwrap();

        let (listener, addr) = listen_available_loopback().await.unwrap();
        let endpoint = format!("127.0.0.1:{}", addr.port());
        let mut book = AddressBook::default();
        book.upsert(tid.as_str(), &endpoint).unwrap();
        book.save(probe.path()).unwrap();

        let root_t = target.path().to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_t).await });
        let probe_session = dial(probe.path(), tid.as_str()).await.unwrap();
        let target_session = accept_task.await.unwrap().unwrap();

        let ch = ReachabilityChallenge::draft(ChallengeDraft {
            target_identity_ref: tid.as_str().into(),
            target_public_key: tpk,
            endpoint: endpoint.clone(),
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-05T13:00:00Z".into(),
        })
        .unwrap()
        .sign_for_node_root(target.path())
        .unwrap();
        let evidence = target_session
            .export_reachability_evidence(&ch, "2026-09-05T12:30:00Z")
            .unwrap();
        let att = ReachabilityAttestation::issue_for_authenticated_session(
            &ch,
            &probe_session,
            "2026-09-05T12:30:00Z",
        )
        .unwrap();
        let result = ReachabilityResult::new(ch, att);

        let mut foreign = ReachabilityLocalState::default();
        let wrong_root = foreign
            .apply_successful_probe(other.path(), &result, &evidence, "2026-09-05T12:30:00Z")
            .unwrap_err();
        assert!(
            matches!(&wrong_root, PeerError::Reachability(msg) if msg.contains("#281")),
            "{wrong_root:?}"
        );

        let mut st = ReachabilityLocalState::default();
        st.apply_successful_probe(target.path(), &result, &evidence, "2026-09-05T12:30:00Z")
            .unwrap();
        let replay_err = st
            .apply_successful_probe(target.path(), &result, &evidence, "2026-09-05T12:30:05Z")
            .unwrap_err();
        assert!(
            matches!(&replay_err, PeerError::Reachability(msg) if msg.contains("replayed")),
            "{replay_err:?}"
        );

        // Stale apply-time / expiry after clearing durable replay (`#281`).
        fs::remove_file(crate::reachability::reachability_replay_path(target.path())).unwrap();
        let mut st2 = ReachabilityLocalState::default();
        let skew = st2
            .apply_successful_probe(target.path(), &result, &evidence, "2026-09-05T12:40:00Z")
            .unwrap_err();
        assert!(matches!(skew, PeerError::ClockSkew), "{skew:?}");

        let mut st3 = ReachabilityLocalState::default();
        let expired = st3
            .apply_successful_probe(target.path(), &result, &evidence, "2026-09-05T14:00:00Z")
            .unwrap_err();
        assert!(
            matches!(&expired, PeerError::Reachability(msg) if msg.contains("apply time")),
            "{expired:?}"
        );
    }

    #[tokio::test]
    async fn apply_rejects_same_identity_foreign_key_package() {
        use crate::{accept, admit_peer_trust, dial, listen_available_loopback, AddressBook};

        // Same identity *name* on two roots, different signing keys (#296).
        let victim = tempdir().unwrap();
        let impostor = tempdir().unwrap();
        let probe = tempdir().unwrap();
        let (vid, _vpk) = write_node(victim.path(), "kb-same", [91u8; 32]);
        let (iid, ipk) = write_node(impostor.path(), "kb-same", [92u8; 32]);
        assert_eq!(vid.as_str(), iid.as_str());
        let (pid, ppk) = write_node(probe.path(), "kb-prb", [93u8; 32]);
        admit_peer_trust(impostor.path(), pid.as_str(), &ppk).unwrap();
        admit_peer_trust(probe.path(), iid.as_str(), &ipk).unwrap();

        let (listener, addr) = listen_available_loopback().await.unwrap();
        let endpoint = format!("127.0.0.1:{}", addr.port());
        let mut book = AddressBook::default();
        book.upsert(iid.as_str(), &endpoint).unwrap();
        book.save(probe.path()).unwrap();

        let root_i = impostor.path().to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_i).await });
        let probe_session = dial(probe.path(), iid.as_str()).await.unwrap();
        let impostor_session = accept_task.await.unwrap().unwrap();

        let ch = ReachabilityChallenge::draft(ChallengeDraft {
            target_identity_ref: iid.as_str().into(),
            target_public_key: ipk,
            endpoint: endpoint.clone(),
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-05T13:00:00Z".into(),
        })
        .unwrap()
        .sign_for_node_root(impostor.path())
        .unwrap();
        let evidence = impostor_session
            .export_reachability_evidence(&ch, "2026-09-05T12:30:00Z")
            .unwrap();
        let att = ReachabilityAttestation::issue_for_authenticated_session(
            &ch,
            &probe_session,
            "2026-09-05T12:30:00Z",
        )
        .unwrap();
        let result = ReachabilityResult::new(ch, att);

        let mut st = ReachabilityLocalState::default();
        let before = st.clone();
        let err = st
            .apply_successful_probe(victim.path(), &result, &evidence, "2026-09-05T12:30:00Z")
            .unwrap_err();
        assert!(
            matches!(&err, PeerError::Reachability(msg) if msg.contains("#296")),
            "{err:?}"
        );
        assert_eq!(st.status, before.status);
        assert_eq!(st.status, ReachabilityStatus::Unknown);
        assert!(!ReachabilityLocalState::path(victim.path()).is_file());
        assert!(!crate::reachability::reachability_replay_path(victim.path()).is_file());
    }

    #[test]
    fn signed_claim_without_session_cannot_apply_direct() {
        let target = tempdir().unwrap();
        let probe = tempdir().unwrap();
        let (tid, tpk) = write_node(target.path(), "no-sess", [83u8; 32]);
        let _ = write_node(probe.path(), "no-sess-p", [84u8; 32]);
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
        assert!(ReachabilityAttestation::issue_for_challenge(
            &ch,
            probe.path(),
            "127.0.0.1:49157",
            "2026-09-05T12:30:00Z",
            true,
        )
        .is_err());
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
        assert_eq!(st.relay_checked_at.as_deref(), Some("2026-09-05T12:00:00Z"));

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
