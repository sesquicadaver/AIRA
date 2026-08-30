//! Durable envelope replay window under `peers/envelope_replay.json` (QUEUE #194).

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::unix_seconds;
use aira_protocol::{admit_envelope, EnvelopeReplayWindow, ProtocolEnvelope, ProtocolError};

use crate::error::PeerError;

/// Path to the node replay window.
pub fn envelope_replay_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("peers").join("envelope_replay.json")
}

fn load_window(root: impl AsRef<Path>) -> Result<EnvelopeReplayWindow, PeerError> {
    let path = envelope_replay_path(&root);
    if !path.exists() {
        return Ok(EnvelopeReplayWindow::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| PeerError::Protocol(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| PeerError::Protocol(e.to_string()))
}

fn save_window(root: impl AsRef<Path>, window: &EnvelopeReplayWindow) -> Result<(), PeerError> {
    let path = envelope_replay_path(&root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PeerError::Protocol(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(window).map_err(|e| PeerError::Protocol(e.to_string()))?;
    fs::write(path, format!("{json}\n")).map_err(|e| PeerError::Protocol(e.to_string()))
}

fn map_admit(err: ProtocolError) -> PeerError {
    match err {
        ProtocolError::Expired => PeerError::Expired,
        ProtocolError::ClockSkew => PeerError::ClockSkew,
        ProtocolError::Duplicate(id) => PeerError::Replay(id.to_string()),
        other => PeerError::Protocol(other.to_string()),
    }
}

/// Signature-verified envelope: reject expired / skewed / replayed `message_id`.
pub fn admit_received_envelope(
    root: impl AsRef<Path>,
    env: &ProtocolEnvelope,
) -> Result<(), PeerError> {
    let now_unix =
        unix_seconds(&aira_object::now()).map_err(|e| PeerError::Protocol(e.to_string()))?;
    let mut window = load_window(&root)?;
    admit_envelope(env, now_unix, &mut window).map_err(map_admit)?;
    save_window(root, &window)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{AiraRef, ContentHash};
    use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
    use tempfile::tempdir;

    fn unsigned(created: &str, expires: Option<&str>, id: &str) -> ProtocolEnvelope {
        let issuer = AiraRef::parse("aira:identity:local-test").unwrap();
        ProtocolEnvelope {
            protocol_id: ProtocolId::Identity,
            protocol_version: "0.1".into(),
            message_type: "peer.ping".into(),
            message_id: AiraRef::parse(id).unwrap(),
            correlation_id: None,
            causal_refs: vec![],
            issuer_identity: issuer.clone(),
            target_scope: ScopeDescriptor::local("replay-unit"),
            policy_refs: vec![],
            payload_hash: ContentHash::sha256_bytes(b"x"),
            payload_ref: None,
            created_at: aira_object::Timestamp::parse(created).unwrap(),
            expires_at: expires.map(str::to_string),
            signature: ProtocolEnvelope::placeholder_signature(&issuer),
        }
    }

    #[test]
    fn admit_received_rejects_clock_skew() {
        let dir = tempdir().unwrap();
        let env = unsigned("2020-01-01T00:00:00Z", None, "aira:message:skew-peer");
        let err = admit_received_envelope(dir.path(), &env).unwrap_err();
        assert!(matches!(err, PeerError::ClockSkew), "{err}");
    }

    #[test]
    fn admit_received_persists_replay() {
        let dir = tempdir().unwrap();
        let env = unsigned(
            aira_object::now().as_str(),
            None,
            "aira:message:persist-peer",
        );
        admit_received_envelope(dir.path(), &env).unwrap();
        assert!(envelope_replay_path(dir.path()).exists());
        let err = admit_received_envelope(dir.path(), &env).unwrap_err();
        assert!(matches!(err, PeerError::Replay(_)), "{err}");
    }
}
