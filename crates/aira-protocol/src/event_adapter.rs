//! Local AIRA-EP Event Protocol adapter (Issue #72).

use std::collections::HashSet;

use aira_event::{EventDescriptor, EventSink, MemoryEventLog};
use aira_object::{AiraRef, ContentHash};
use serde_json::to_vec;

use crate::envelope::{
    local_identity, local_signature, mvp_timestamp, ProtocolEnvelope, ProtocolError, ProtocolId,
    ProtocolResponse, ProtocolStatus, ScopeDescriptor,
};

/// Supported local event protocol version.
pub const EP_VERSION: &str = "0.1";

/// Local AIRA-EP adapter over an in-process event log.
pub struct EventProtocolAdapter {
    log: MemoryEventLog,
    /// Idempotency keyed by event_id.
    seen_events: HashSet<String>,
    seq: u64,
}

impl Default for EventProtocolAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventProtocolAdapter {
    pub fn new() -> Self {
        Self {
            log: MemoryEventLog::new(),
            seen_events: HashSet::new(),
            seq: 1,
        }
    }

    pub fn events(&self) -> &[EventDescriptor] {
        self.log.all()
    }

    /// Publish a local event via protocol envelope.
    pub fn publish_event(
        &mut self,
        event: EventDescriptor,
        protocol_version: &str,
    ) -> Result<(ProtocolEnvelope, ProtocolResponse), ProtocolError> {
        let envelope = self.wrap_event(&event, protocol_version)?;

        if protocol_version != EP_VERSION {
            let resp = self.response(
                &event.event_id,
                ProtocolStatus::UnsupportedVersion,
                Some(envelope.message_id.as_str()),
            )?;
            return Ok((envelope, resp));
        }

        envelope.validate_signature()?;

        let event_key = event.event_id.as_str().to_string();
        if self.seen_events.contains(&event_key) {
            let resp = self.response(
                &event.event_id,
                ProtocolStatus::Accepted,
                Some(envelope.message_id.as_str()),
            )?;
            return Ok((envelope, resp));
        }

        self.log
            .append(event.clone())
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
        self.seen_events.insert(event_key);

        let resp = self.response(
            &event.event_id,
            ProtocolStatus::Accepted,
            Some(envelope.message_id.as_str()),
        )?;
        Ok((envelope, resp))
    }

    fn wrap_event(
        &mut self,
        event: &EventDescriptor,
        protocol_version: &str,
    ) -> Result<ProtocolEnvelope, ProtocolError> {
        let payload = to_vec(event).map_err(|e| ProtocolError::Storage(e.to_string()))?;
        let hash = ContentHash::sha256_bytes(&payload);
        self.seq += 1;
        let message_id = AiraRef::parse(format!("aira:message:ep{}", self.seq))
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
        Ok(ProtocolEnvelope {
            protocol_id: ProtocolId::Event,
            protocol_version: protocol_version.into(),
            message_type: "EventPublish".into(),
            message_id,
            correlation_id: Some(event.event_id.as_str().to_string()),
            causal_refs: event.causal_refs.clone(),
            issuer_identity: local_identity(),
            target_scope: ScopeDescriptor::local("event-protocol"),
            policy_refs: event.policy_refs.clone(),
            payload_hash: hash,
            payload_ref: Some(format!("event:{}", event.event_id)),
            created_at: mvp_timestamp(),
            expires_at: None,
            signature: local_signature(),
        })
    }

    fn response(
        &mut self,
        event_id: &AiraRef,
        status: ProtocolStatus,
        correlation: Option<&str>,
    ) -> Result<ProtocolResponse, ProtocolError> {
        self.seq += 1;
        let message_id = AiraRef::parse(format!("aira:message:epresp{}", self.seq))
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
        Ok(ProtocolResponse {
            message_id,
            correlation_id: correlation.map(|s| s.to_string()),
            status,
            reason_refs: vec![event_id.clone()],
            created_at: mvp_timestamp(),
            signature: local_signature(),
        })
    }
}
