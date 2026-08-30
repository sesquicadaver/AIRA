//! Envelope freshness and message-id replay window (QUEUE #194).

use aira_object::{unix_seconds, unix_seconds_str};
use serde::{Deserialize, Serialize};

use crate::envelope::{ProtocolEnvelope, ProtocolError};

/// Default |created_at − now| bound (seconds).
pub const DEFAULT_MAX_SKEW_SECS: i64 = 300;

/// Default replay memory TTL (seconds).
pub const DEFAULT_REPLAY_TTL_SECS: i64 = 600;

/// Default max retained message ids.
pub const DEFAULT_REPLAY_CAP: usize = 4096;

/// Policy window for peer-receive admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeAdmitPolicy {
    pub max_skew_secs: i64,
    pub replay_ttl_secs: i64,
    pub replay_cap: usize,
}

impl Default for EnvelopeAdmitPolicy {
    fn default() -> Self {
        Self {
            max_skew_secs: DEFAULT_MAX_SKEW_SECS,
            replay_ttl_secs: DEFAULT_REPLAY_TTL_SECS,
            replay_cap: DEFAULT_REPLAY_CAP,
        }
    }
}

/// One seen message id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeReplayEntry {
    pub message_id: String,
    pub seen_at_unix: i64,
}

/// Sliding message-id window (in-memory; peer crate may persist).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeReplayWindow {
    #[serde(default)]
    pub entries: Vec<EnvelopeReplayEntry>,
}

impl EnvelopeReplayWindow {
    fn prune(&mut self, now_unix: i64, ttl: i64, cap: usize) {
        self.entries
            .retain(|e| now_unix.saturating_sub(e.seen_at_unix) <= ttl);
        if self.entries.len() > cap {
            let drop_n = self.entries.len() - cap;
            self.entries.drain(0..drop_n);
        }
    }

    /// Admit `env` at `now_unix` or reject expired / skewed / replayed ids.
    pub fn admit(
        &mut self,
        env: &ProtocolEnvelope,
        now_unix: i64,
        policy: &EnvelopeAdmitPolicy,
    ) -> Result<(), ProtocolError> {
        let created =
            unix_seconds(&env.created_at).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        if let Some(exp) = env.expires_at.as_deref().filter(|s| !s.is_empty()) {
            let exp_unix =
                unix_seconds_str(exp).map_err(|e| ProtocolError::Schema(e.to_string()))?;
            if now_unix >= exp_unix {
                return Err(ProtocolError::Expired);
            }
        }
        if (now_unix - created).abs() > policy.max_skew_secs {
            return Err(ProtocolError::ClockSkew);
        }
        let id = env.message_id.as_str();
        self.prune(now_unix, policy.replay_ttl_secs, policy.replay_cap);
        if self.entries.iter().any(|e| e.message_id == id) {
            return Err(ProtocolError::Duplicate(env.message_id.clone()));
        }
        self.entries.push(EnvelopeReplayEntry {
            message_id: id.to_string(),
            seen_at_unix: now_unix,
        });
        self.prune(now_unix, policy.replay_ttl_secs, policy.replay_cap);
        Ok(())
    }
}

/// Convenience: admit with [`EnvelopeAdmitPolicy::default`].
pub fn admit_envelope(
    env: &ProtocolEnvelope,
    now_unix: i64,
    window: &mut EnvelopeReplayWindow,
) -> Result<(), ProtocolError> {
    window.admit(env, now_unix, &EnvelopeAdmitPolicy::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{unix_seconds, AiraRef, ContentHash, Timestamp};

    use crate::envelope::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};

    fn now_unix() -> i64 {
        unix_seconds(&Timestamp::parse("2026-08-30T12:00:00Z").unwrap()).unwrap()
    }

    fn env_at(id: &str, created: &str, expires: Option<&str>) -> ProtocolEnvelope {
        let issuer = AiraRef::parse("aira:identity:local-test").unwrap();
        ProtocolEnvelope {
            protocol_id: ProtocolId::Identity,
            protocol_version: "0.1".into(),
            message_type: "peer.ping".into(),
            message_id: AiraRef::parse(id).unwrap(),
            correlation_id: None,
            causal_refs: vec![],
            issuer_identity: issuer.clone(),
            target_scope: ScopeDescriptor::local("admit-test"),
            policy_refs: vec![],
            payload_hash: ContentHash::sha256_bytes(b"x"),
            payload_ref: None,
            created_at: Timestamp::parse(created).unwrap(),
            expires_at: expires.map(str::to_string),
            signature: ProtocolEnvelope::placeholder_signature(&issuer),
        }
    }

    #[test]
    fn fresh_envelope_is_admitted() {
        let mut w = EnvelopeReplayWindow::default();
        let env = env_at(
            "aira:message:fresh1",
            "2026-08-30T12:00:00Z",
            Some("2026-08-30T12:04:00Z"),
        );
        admit_envelope(&env, now_unix(), &mut w).unwrap();
    }

    #[test]
    fn expired_envelope_is_rejected() {
        let mut w = EnvelopeReplayWindow::default();
        let env = env_at(
            "aira:message:exp1",
            "2026-08-30T12:00:00Z",
            Some("2026-08-30T11:59:59Z"),
        );
        assert!(matches!(
            admit_envelope(&env, now_unix(), &mut w),
            Err(ProtocolError::Expired)
        ));
    }

    #[test]
    fn skewed_created_at_is_rejected() {
        let mut w = EnvelopeReplayWindow::default();
        let env = env_at("aira:message:skew1", "2020-01-01T00:00:00Z", None);
        assert!(matches!(
            admit_envelope(&env, now_unix(), &mut w),
            Err(ProtocolError::ClockSkew)
        ));
    }

    #[test]
    fn duplicate_message_id_is_rejected_within_window() {
        let mut w = EnvelopeReplayWindow::default();
        let env = env_at("aira:message:dup1", "2026-08-30T12:00:00Z", None);
        admit_envelope(&env, now_unix(), &mut w).unwrap();
        assert!(matches!(
            admit_envelope(&env, now_unix(), &mut w),
            Err(ProtocolError::Duplicate(_))
        ));
    }

    #[test]
    fn replay_entry_expires_after_ttl() {
        let mut w = EnvelopeReplayWindow::default();
        let policy = EnvelopeAdmitPolicy {
            max_skew_secs: 10_000,
            replay_ttl_secs: 10,
            replay_cap: 16,
        };
        let env = env_at("aira:message:ttl1", "2026-08-30T12:00:00Z", None);
        w.admit(&env, now_unix(), &policy).unwrap();
        assert!(w.admit(&env, now_unix() + 11, &policy).is_ok());
    }
}
