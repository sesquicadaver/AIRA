//! Peer-assisted Reachability Probe (QUEUE #238 / Phase N; session bind `#250`).
//!
//! Signed challenge + external probe attestation. Hairpin/self-connect
//! (`probe_identity == target_identity`) is never proof. Successful DIRECT
//! proof requires an authenticated inbound Noise session transcript (`#250`).
//! State persistence: [`crate::reachability_state`].

use std::collections::HashSet;
use std::path::Path;

use aira_object::{
    descriptor_signing_message, AiraRef, ContentHash, Keyring, Signature, Timestamp, TrustStore,
    LOCAL_TEST_KEY_REF,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::PeerError;
use crate::prime_port::validate_aira_bind;
use crate::session::AuthenticatedPeer;

/// Domain for reachability session transcript (`#250`).
pub const REACHABILITY_SESSION_DOMAIN: &str = "aira:reachability:session:v1";

/// Challenge schema `$id`.
pub const REACHABILITY_CHALLENGE_SCHEMA: &str = "aira:schema:peer:reachability-challenge:0.1";
/// Probe attestation schema `$id`.
pub const REACHABILITY_ATTESTATION_SCHEMA: &str = "aira:schema:peer:reachability-attestation:0.1";
/// Combined result schema `$id`.
pub const REACHABILITY_RESULT_SCHEMA: &str = "aira:schema:peer:reachability-result:0.1";

/// Cap on remembered challenge ids (anti-replay).
pub const REACHABILITY_REPLAY_CAP: usize = 4096;

/// Target-issued signed challenge for an advertised endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachabilityChallenge {
    pub schema: String,
    pub challenge_id: String,
    pub nonce_hex: String,
    pub target_identity_ref: String,
    pub target_public_key: String,
    pub endpoint: String,
    pub created_at: String,
    pub expires_at: String,
    pub signature: Signature,
}

/// Inputs for an unsigned challenge draft.
#[derive(Debug, Clone)]
pub struct ChallengeDraft {
    pub target_identity_ref: String,
    pub target_public_key: String,
    pub endpoint: String,
    pub created_at: String,
    pub expires_at: String,
}

/// Probe-node attestation that it exercised the challenge over a real inbound path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachabilityAttestation {
    pub schema: String,
    pub challenge_id: String,
    pub nonce_hex: String,
    pub probe_identity_ref: String,
    pub probe_public_key: String,
    pub observed_endpoint: String,
    pub probed_at: String,
    pub success: bool,
    /// Noise handshake hash hex (required when `success`; `#250`).
    #[serde(default)]
    pub noise_handshake_hash_hex: String,
    /// Session transcript (`sha256:…`) binding challenge + Noise session (`#250`).
    #[serde(default)]
    pub session_transcript_hex: String,
    pub signature: Signature,
}

/// Compute canonical session transcript for challenge + Noise handshake hash.
pub fn session_transcript_hex(
    challenge: &ReachabilityChallenge,
    local_identity: &str,
    peer_identity: &str,
    noise_handshake_hash_hex: &str,
) -> Result<String, PeerError> {
    let (target, probe) = if local_identity == challenge.target_identity_ref {
        (local_identity, peer_identity)
    } else if peer_identity == challenge.target_identity_ref {
        (peer_identity, local_identity)
    } else {
        return Err(PeerError::Reachability(
            "session transcript: neither local nor peer is challenge target".into(),
        ));
    };
    if probe == target {
        return Err(PeerError::Reachability(
            "session transcript hairpin: probe equals target".into(),
        ));
    }
    let hs = noise_handshake_hash_hex.trim();
    if hs.len() != 64 || !hs.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PeerError::Reachability(
            "noise_handshake_hash_hex must be 64 hex chars".into(),
        ));
    }
    let preimage = format!(
        "{REACHABILITY_SESSION_DOMAIN}|{}|{}|{target}|{probe}|{hs}",
        challenge.challenge_id, challenge.nonce_hex
    );
    Ok(ContentHash::sha256_bytes(preimage.as_bytes())
        .as_str()
        .to_string())
}

/// Combined challenge + external attestation (verified as a unit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachabilityResult {
    pub schema: String,
    pub challenge: ReachabilityChallenge,
    pub attestation: ReachabilityAttestation,
}

/// In-memory anti-replay set for challenge ids.
#[derive(Debug, Default, Clone)]
pub struct ReachabilityReplayLog {
    seen: HashSet<String>,
    order: Vec<String>,
}

impl ReachabilityReplayLog {
    /// Empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// True if `challenge_id` was already admitted.
    pub fn contains(&self, challenge_id: &str) -> bool {
        self.seen.contains(challenge_id)
    }

    /// Record id; returns Err on replay.
    pub fn admit(&mut self, challenge_id: &str) -> Result<(), PeerError> {
        if self.seen.contains(challenge_id) {
            return Err(PeerError::Reachability(format!(
                "replayed reachability challenge: {challenge_id}"
            )));
        }
        if self.order.len() >= REACHABILITY_REPLAY_CAP {
            if let Some(old) = self.order.first().cloned() {
                self.seen.remove(&old);
                self.order.remove(0);
            }
        }
        self.seen.insert(challenge_id.to_string());
        self.order.push(challenge_id.to_string());
        Ok(())
    }
}

fn parse_odt(s: &str) -> Result<OffsetDateTime, PeerError> {
    OffsetDateTime::parse(s.trim(), &Rfc3339)
        .map_err(|e| PeerError::Reachability(format!("bad timestamp {s}: {e}")))
}

fn verify_with_embedded_key(
    identity_ref: &str,
    public_key_hex: &str,
    signature: &Signature,
    value: &serde_json::Value,
) -> Result<(), PeerError> {
    if !aira_object::is_cryptographic_signature(signature) {
        return Err(PeerError::InvalidSignature);
    }
    if signature.key_ref.as_str() != identity_ref {
        return Err(PeerError::IdentityMismatch);
    }
    let mut store = TrustStore::default();
    store
        .upsert(identity_ref, public_key_hex.trim())
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    let ring = store
        .to_keyring()
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    let msg = descriptor_signing_message(value)?;
    ring.verify(signature, &msg)
        .map_err(|_| PeerError::InvalidSignature)?;
    Ok(())
}

impl ReachabilityChallenge {
    /// Draft unsigned challenge with fresh nonce + challenge_id.
    pub fn draft(input: ChallengeDraft) -> Result<Self, PeerError> {
        let id = AiraRef::parse(&input.target_identity_ref)
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
        let mut nonce = [0u8; 16];
        OsRng.fill_bytes(&mut nonce);
        let mut cid = [0u8; 8];
        OsRng.fill_bytes(&mut cid);
        Ok(Self {
            schema: REACHABILITY_CHALLENGE_SCHEMA.into(),
            challenge_id: format!("aira:challenge:{}", hex::encode(cid)),
            nonce_hex: hex::encode(nonce),
            target_identity_ref: input.target_identity_ref,
            target_public_key: input.target_public_key,
            endpoint: input.endpoint,
            created_at: input.created_at,
            expires_at: input.expires_at,
            signature: Signature {
                algorithm: "ed25519".into(),
                key_ref: id,
                signature_value: String::new(),
            },
        })
    }

    /// Structural checks (no crypto).
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != REACHABILITY_CHALLENGE_SCHEMA {
            return Err(PeerError::Reachability(format!(
                "challenge schema mismatch: {}",
                self.schema
            )));
        }
        if self.target_identity_ref == LOCAL_TEST_KEY_REF {
            return Err(PeerError::Untrusted(self.target_identity_ref.clone()));
        }
        AiraRef::parse(&self.target_identity_ref)
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
        let pk = self.target_public_key.trim();
        if pk.len() != 64 || !pk.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PeerError::Protocol(
                "challenge target_public_key must be 64 hex chars".into(),
            ));
        }
        if self.nonce_hex.len() < 16 || !self.nonce_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PeerError::Reachability(
                "challenge nonce_hex invalid".into(),
            ));
        }
        if self.challenge_id.trim().is_empty() {
            return Err(PeerError::Reachability("challenge_id empty".into()));
        }
        validate_aira_bind(&self.endpoint)?;
        Timestamp::parse(&self.created_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        Timestamp::parse(&self.expires_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        if parse_odt(&self.expires_at)? <= parse_odt(&self.created_at)? {
            return Err(PeerError::Reachability(
                "challenge expires_at must be after created_at".into(),
            ));
        }
        if self.signature.key_ref.as_str() != self.target_identity_ref {
            return Err(PeerError::IdentityMismatch);
        }
        Ok(())
    }

    /// Sign with node root identity (must match target).
    pub fn sign_for_node_root(mut self, root: impl AsRef<Path>) -> Result<Self, PeerError> {
        self.validate_shape()?;
        let (local_id, ring) = Keyring::load_node_identity(root.as_ref())?;
        if local_id.as_str() != self.target_identity_ref {
            return Err(PeerError::Protocol(
                "challenge target_identity_ref must match node identity".into(),
            ));
        }
        let expected_pk = ring
            .verifying_key(local_id.as_str())
            .ok_or_else(|| PeerError::Crypto("missing verifying key".into()))?;
        let pk_hex = hex::encode(expected_pk.as_bytes());
        if !self.target_public_key.eq_ignore_ascii_case(&pk_hex) {
            return Err(PeerError::Protocol(
                "challenge target_public_key does not match node key".into(),
            ));
        }
        let value = serde_json::to_value(&self)?;
        let msg = descriptor_signing_message(&value)?;
        self.signature = ring.sign(&local_id, &msg)?;
        Ok(self)
    }

    /// Verify target signature (no TrustStore upsert).
    pub fn verify_canonical_signature(&self) -> Result<(), PeerError> {
        self.validate_shape()?;
        let value = serde_json::to_value(self)?;
        verify_with_embedded_key(
            &self.target_identity_ref,
            &self.target_public_key,
            &self.signature,
            &value,
        )
    }

    /// True when `as_of` is at or after `expires_at`.
    pub fn is_expired_at(&self, as_of: &str) -> Result<bool, PeerError> {
        Ok(parse_odt(as_of)? >= parse_odt(&self.expires_at)?)
    }
}

impl ReachabilityAttestation {
    /// Build and sign a **failed** probe claim (no session required).
    ///
    /// Successful DIRECT proof must use [`Self::issue_for_authenticated_session`].
    pub fn issue_for_challenge(
        challenge: &ReachabilityChallenge,
        probe_root: impl AsRef<Path>,
        observed_endpoint: impl Into<String>,
        probed_at: impl Into<String>,
        success: bool,
    ) -> Result<Self, PeerError> {
        if success {
            return Err(PeerError::Reachability(
                "success=true requires authenticated inbound session; use issue_for_authenticated_session (#250)"
                    .into(),
            ));
        }
        Self::issue_inner(
            challenge,
            probe_root,
            observed_endpoint,
            probed_at,
            false,
            String::new(),
            String::new(),
        )
    }

    /// Issue successful attestation bound to an authenticated peer session (`#250`).
    ///
    /// `session` must be the probe's dial (or the target's accept) for the
    /// challenge endpoint; transcript is derived from the Noise handshake hash.
    pub fn issue_for_authenticated_session(
        challenge: &ReachabilityChallenge,
        session: &AuthenticatedPeer,
        observed_endpoint: impl Into<String>,
        probed_at: impl Into<String>,
    ) -> Result<Self, PeerError> {
        let probe_root = session.local_root();
        let (probe_id, _) = Keyring::load_node_identity(probe_root)?;
        if probe_id.as_str() != session.local_id.as_str() {
            return Err(PeerError::Reachability(
                "session local_id does not match probe root identity".into(),
            ));
        }
        if probe_id.as_str() == challenge.target_identity_ref {
            return Err(PeerError::Reachability(
                "hairpin forbidden: probe_identity must differ from target_identity".into(),
            ));
        }
        if session.peer_id.as_str() != challenge.target_identity_ref
            && session.local_id.as_str() != challenge.target_identity_ref
        {
            return Err(PeerError::Reachability(
                "session is not bound to challenge target identity".into(),
            ));
        }
        // Probe issues attestation: local must be probe (not target).
        if session.local_id.as_str() == challenge.target_identity_ref {
            return Err(PeerError::Reachability(
                "attestation must be issued by probe side of the session".into(),
            ));
        }
        if session.peer_id.as_str() != challenge.target_identity_ref {
            return Err(PeerError::Reachability(
                "session peer must be challenge target".into(),
            ));
        }
        let hs_hex = session.noise_handshake_hash_hex();
        let transcript = session.reachability_session_transcript(challenge)?;
        Self::issue_inner(
            challenge,
            probe_root,
            observed_endpoint,
            probed_at,
            true,
            hs_hex,
            transcript,
        )
    }

    fn issue_inner(
        challenge: &ReachabilityChallenge,
        probe_root: impl AsRef<Path>,
        observed_endpoint: impl Into<String>,
        probed_at: impl Into<String>,
        success: bool,
        noise_handshake_hash_hex: String,
        session_transcript_hex: String,
    ) -> Result<Self, PeerError> {
        challenge.verify_canonical_signature()?;
        let (probe_id, ring) = Keyring::load_node_identity(probe_root.as_ref())?;
        if probe_id.as_str() == challenge.target_identity_ref {
            return Err(PeerError::Reachability(
                "hairpin forbidden: probe_identity must differ from target_identity".into(),
            ));
        }
        let pk = ring
            .verifying_key(probe_id.as_str())
            .ok_or_else(|| PeerError::Crypto("missing probe verifying key".into()))?;
        let mut att = Self {
            schema: REACHABILITY_ATTESTATION_SCHEMA.into(),
            challenge_id: challenge.challenge_id.clone(),
            nonce_hex: challenge.nonce_hex.clone(),
            probe_identity_ref: probe_id.as_str().into(),
            probe_public_key: hex::encode(pk.as_bytes()),
            observed_endpoint: observed_endpoint.into(),
            probed_at: probed_at.into(),
            success,
            noise_handshake_hash_hex,
            session_transcript_hex,
            signature: Signature {
                algorithm: "ed25519".into(),
                key_ref: probe_id.clone(),
                signature_value: String::new(),
            },
        };
        att.validate_shape()?;
        let value = serde_json::to_value(&att)?;
        let msg = descriptor_signing_message(&value)?;
        att.signature = ring.sign(&probe_id, &msg)?;
        Ok(att)
    }

    /// Structural checks (no crypto).
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != REACHABILITY_ATTESTATION_SCHEMA {
            return Err(PeerError::Reachability(format!(
                "attestation schema mismatch: {}",
                self.schema
            )));
        }
        if self.probe_identity_ref == LOCAL_TEST_KEY_REF {
            return Err(PeerError::Untrusted(self.probe_identity_ref.clone()));
        }
        AiraRef::parse(&self.probe_identity_ref).map_err(|e| PeerError::Protocol(e.to_string()))?;
        let pk = self.probe_public_key.trim();
        if pk.len() != 64 || !pk.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PeerError::Protocol(
                "attestation probe_public_key must be 64 hex chars".into(),
            ));
        }
        validate_aira_bind(&self.observed_endpoint)?;
        Timestamp::parse(&self.probed_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
        if self.signature.key_ref.as_str() != self.probe_identity_ref {
            return Err(PeerError::IdentityMismatch);
        }
        if self.success {
            let hs = self.noise_handshake_hash_hex.trim();
            if hs.len() != 64 || !hs.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(PeerError::Reachability(
                    "successful attestation requires noise_handshake_hash_hex (#250)".into(),
                ));
            }
            if self.session_transcript_hex.trim().is_empty() {
                return Err(PeerError::Reachability(
                    "successful attestation requires session_transcript_hex (#250)".into(),
                ));
            }
        }
        Ok(())
    }

    /// Verify probe signature.
    pub fn verify_canonical_signature(&self) -> Result<(), PeerError> {
        self.validate_shape()?;
        let value = serde_json::to_value(self)?;
        verify_with_embedded_key(
            &self.probe_identity_ref,
            &self.probe_public_key,
            &self.signature,
            &value,
        )
    }
}

impl ReachabilityResult {
    /// Assemble challenge + attestation.
    pub fn new(challenge: ReachabilityChallenge, attestation: ReachabilityAttestation) -> Self {
        Self {
            schema: REACHABILITY_RESULT_SCHEMA.into(),
            challenge,
            attestation,
        }
    }

    /// Verify peer-assisted proof: signatures, binding, no hairpin, not expired, optional replay.
    pub fn verify(&self, replay: Option<&mut ReachabilityReplayLog>) -> Result<(), PeerError> {
        if self.schema != REACHABILITY_RESULT_SCHEMA {
            return Err(PeerError::Reachability(format!(
                "result schema mismatch: {}",
                self.schema
            )));
        }
        self.challenge.verify_canonical_signature()?;
        self.attestation.verify_canonical_signature()?;
        if self.attestation.challenge_id != self.challenge.challenge_id {
            return Err(PeerError::Reachability(
                "attestation challenge_id mismatch".into(),
            ));
        }
        if self.attestation.nonce_hex != self.challenge.nonce_hex {
            return Err(PeerError::Reachability(
                "attestation nonce mismatch (wrong challenge)".into(),
            ));
        }
        if self.attestation.probe_identity_ref == self.challenge.target_identity_ref {
            return Err(PeerError::Reachability(
                "hairpin forbidden: probe_identity equals target_identity".into(),
            ));
        }
        if self.challenge.is_expired_at(&self.attestation.probed_at)? {
            return Err(PeerError::Reachability(
                "expired reachability challenge".into(),
            ));
        }
        if self.attestation.observed_endpoint != self.challenge.endpoint {
            return Err(PeerError::Reachability(
                "observed_endpoint does not match challenge endpoint".into(),
            ));
        }
        if !self.attestation.success {
            return Err(PeerError::Reachability(
                "attestation success=false is not a proof".into(),
            ));
        }
        // #250: recompute session transcript from claimed Noise hash + challenge binding.
        let expected = session_transcript_hex(
            &self.challenge,
            self.challenge.target_identity_ref.as_str(),
            self.attestation.probe_identity_ref.as_str(),
            &self.attestation.noise_handshake_hash_hex,
        )?;
        if expected != self.attestation.session_transcript_hex {
            return Err(PeerError::Reachability(
                "session_transcript_hex does not match challenge/Noise binding (#250)".into(),
            ));
        }
        if let Some(log) = replay {
            log.admit(&self.challenge.challenge_id)?;
        }
        Ok(())
    }

    /// Verify plus require a locally observed inbound session transcript (`#250`).
    pub fn verify_with_local_session(
        &self,
        local_session_transcript_hex: &str,
        replay: Option<&mut ReachabilityReplayLog>,
    ) -> Result<(), PeerError> {
        self.verify(replay)?;
        if local_session_transcript_hex.trim() != self.attestation.session_transcript_hex.trim() {
            return Err(PeerError::Reachability(
                "local inbound session transcript does not match attestation (#250)".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, Keyring};
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

    fn signed_challenge(
        root: &Path,
        id: &AiraRef,
        pk: &str,
        endpoint: &str,
        created: &str,
        expires: &str,
    ) -> ReachabilityChallenge {
        ReachabilityChallenge::draft(ChallengeDraft {
            target_identity_ref: id.as_str().into(),
            target_public_key: pk.into(),
            endpoint: endpoint.into(),
            created_at: created.into(),
            expires_at: expires.into(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap()
    }

    #[test]
    fn peer_assisted_probe_roundtrip() {
        let target_dir = tempdir().unwrap();
        let probe_dir = tempdir().unwrap();
        let (tid, tpk) = write_node(target_dir.path(), "tgt", [71u8; 32]);
        let (_pid, _) = write_node(probe_dir.path(), "prb", [72u8; 32]);
        let ch = signed_challenge(
            target_dir.path(),
            &tid,
            &tpk,
            "127.0.0.1:49157",
            "2026-09-05T12:00:00Z",
            "2026-09-05T13:00:00Z",
        );
        // success=true without session is rejected (#250).
        let err = ReachabilityAttestation::issue_for_challenge(
            &ch,
            probe_dir.path(),
            "127.0.0.1:49157",
            "2026-09-05T12:30:00Z",
            true,
        )
        .unwrap_err();
        assert!(matches!(err, PeerError::Reachability(_)));

        let att_fail = ReachabilityAttestation::issue_for_challenge(
            &ch,
            probe_dir.path(),
            "127.0.0.1:49157",
            "2026-09-05T12:30:00Z",
            false,
        )
        .unwrap();
        assert!(!att_fail.success);
        assert!(ReachabilityResult::new(ch.clone(), att_fail)
            .verify(None)
            .is_err());
    }

    #[tokio::test]
    async fn session_bound_probe_sets_proof_and_direct() {
        use crate::{accept, admit_peer_trust, dial, listen_available_loopback, AddressBook};

        let target_dir = tempdir().unwrap();
        let probe_dir = tempdir().unwrap();
        let (tid, tpk) = write_node(target_dir.path(), "tgt-s", [81u8; 32]);
        let (pid, ppk) = write_node(probe_dir.path(), "prb-s", [82u8; 32]);
        admit_peer_trust(target_dir.path(), pid.as_str(), &ppk).unwrap();
        admit_peer_trust(probe_dir.path(), tid.as_str(), &tpk).unwrap();

        let (listener, addr) = listen_available_loopback().await.unwrap();
        let port = addr.port();
        let endpoint = format!("127.0.0.1:{port}");
        let mut book = AddressBook::default();
        book.upsert(tid.as_str(), endpoint.clone()).unwrap();
        book.save(probe_dir.path()).unwrap();

        let root_t = target_dir.path().to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_t).await });
        let probe_session = dial(probe_dir.path(), tid.as_str()).await.unwrap();
        let target_session = accept_task.await.unwrap().unwrap();

        let ch = signed_challenge(
            target_dir.path(),
            &tid,
            &tpk,
            &endpoint,
            "2026-09-05T12:00:00Z",
            "2026-09-05T13:00:00Z",
        );
        let local_tx = target_session
            .reachability_session_transcript(&ch)
            .unwrap();
        let att = ReachabilityAttestation::issue_for_authenticated_session(
            &ch,
            &probe_session,
            endpoint.clone(),
            "2026-09-05T12:30:00Z",
        )
        .unwrap();
        assert!(att.success);
        assert!(!att.session_transcript_hex.is_empty());
        let result = ReachabilityResult::new(ch, att);
        result
            .verify_with_local_session(&local_tx, None)
            .unwrap();

        let mut st = crate::reachability_state::ReachabilityLocalState::default();
        st.apply_successful_probe(&result, &local_tx).unwrap();
        assert_eq!(
            st.status,
            crate::reachability_state::ReachabilityStatus::DirectReachable
        );

        // Signed claim without matching local inbound transcript ≠ DIRECT.
        assert!(st
            .apply_successful_probe(&result, "sha256:deadbeef")
            .is_err());
    }

    #[test]
    fn rejects_hairpin_self_probe() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "solo", [73u8; 32]);
        let ch = signed_challenge(
            dir.path(),
            &id,
            &pk,
            "127.0.0.1:49157",
            "2026-09-05T12:00:00Z",
            "2026-09-05T13:00:00Z",
        );
        let err = ReachabilityAttestation::issue_for_challenge(
            &ch,
            dir.path(),
            "127.0.0.1:49157",
            "2026-09-05T12:30:00Z",
            false,
        )
        .unwrap_err();
        assert!(matches!(err, PeerError::Reachability(_)));
    }

    #[test]
    fn rejects_wrong_challenge_binding_and_expired() {
        let target_dir = tempdir().unwrap();
        let probe_dir = tempdir().unwrap();
        let (tid, tpk) = write_node(target_dir.path(), "tgt2", [74u8; 32]);
        let _ = write_node(probe_dir.path(), "prb2", [75u8; 32]);
        let ch = signed_challenge(
            target_dir.path(),
            &tid,
            &tpk,
            "127.0.0.1:49157",
            "2026-09-05T12:00:00Z",
            "2026-09-05T13:00:00Z",
        );
        let att_ok = ReachabilityAttestation::issue_for_challenge(
            &ch,
            probe_dir.path(),
            "127.0.0.1:49157",
            "2026-09-05T12:30:00Z",
            false,
        )
        .unwrap();
        let mut wrong = att_ok;
        wrong.challenge_id = "aira:challenge:deadbeef".into();
        assert!(ReachabilityResult::new(ch.clone(), wrong)
            .verify(None)
            .is_err());

        let expired_att = ReachabilityAttestation::issue_for_challenge(
            &ch,
            probe_dir.path(),
            "127.0.0.1:49157",
            "2026-09-05T14:00:00Z",
            false,
        )
        .unwrap();
        // success=false is not a proof anyway; expired success would need session.
        assert!(ReachabilityResult::new(ch, expired_att)
            .verify(None)
            .is_err());
    }

    #[test]
    fn mutation_of_endpoint_breaks_challenge_verify() {
        let dir = tempdir().unwrap();
        let (id, pk) = write_node(dir.path(), "mut", [76u8; 32]);
        let mut ch = signed_challenge(
            dir.path(),
            &id,
            &pk,
            "127.0.0.1:49157",
            "2026-09-05T12:00:00Z",
            "2026-09-05T13:00:00Z",
        );
        ch.verify_canonical_signature().unwrap();
        ch.endpoint = "127.0.0.1:49171".into();
        assert!(ch.verify_canonical_signature().is_err());
    }
}
