//! AIRA local C2 protocol surface (Issue Set Epic 10 / #71–#75).
//!
//! Protocol Envelope + local adapters for Event / Artifact / Identity / Discovery.

mod admit;
mod artifact_adapter;
mod capability_ad;
mod crp;
mod discovery;
mod envelope;
mod event_adapter;
mod federation;
mod identity;
mod settlement;

pub use admit::{
    admit_envelope, EnvelopeAdmitPolicy, EnvelopeReplayEntry, EnvelopeReplayWindow,
    DEFAULT_MAX_SKEW_SECS, DEFAULT_REPLAY_CAP, DEFAULT_REPLAY_TTL_SECS,
};
pub use artifact_adapter::{ArtifactOpResult, ArtifactProtocolAdapter, AP_VERSION};
pub use capability_ad::{
    CapabilityAdBody, CapabilityAdvertisement, CapabilityAdvertisementStore, ValidityWindow,
    CAPABILITY_AD_STORE_SCHEMA,
};
pub use crp::{
    CapabilityChainHop, CrpBindOutcome, CrpRouteCandidate, CrpRouteConstraints, CrpRouteOutcome,
    CrpRouteRequest, LocalCrpAdapter, CRP_BIND_ACTION, CRP_VERSION,
};
pub use discovery::{
    CapabilityDescriptor, DiscoveryHit, DiscoveryRegistry, DISCOVERY_REGISTRY_SCHEMA,
};
pub use envelope::{
    local_identity, local_signature, mvp_timestamp, ProtocolEnvelope, ProtocolError, ProtocolId,
    ProtocolResponse, ProtocolStatus, ScopeDescriptor,
};
pub use event_adapter::{EventProtocolAdapter, EP_VERSION};
pub use federation::{
    check_federation_transfer, descriptor_canonical_bytes, federation_io_policy_path,
    join_federation, leave_federation, load_federation_io_policy, load_federation_membership,
    membership_path, save_federation_io_policy, verify_federation_descriptor, FederationDescriptor,
    FederationError, FederationIoPolicy, FederationMembership, FederationTransferKind, JoinOutcome,
    LeaveOutcome, TransferCheckOutcome, FEDERATION_DESCRIPTOR_DOMAIN, FEDERATION_IO_POLICY_SCHEMA,
    FEDERATION_MEMBERSHIP_SCHEMA,
};
pub use identity::{IdentityDescriptor, IdentityType, PublicKeyEntry};
pub use settlement::{
    parse_receipt_privacy_checked, validate_settlement_privacy, ContributionDescriptor,
    SettlementReceipt, SettlementReceiptStore, SETTLEMENT_PRIVACY_FORBIDDEN_KEYS,
    SETTLEMENT_RECEIPTS_REL, SETTLEMENT_RECEIPTS_STORE_SCHEMA,
};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::ArtifactType;
    use aira_csu::support::{make_artifact, make_event};
    use aira_event::EventType;
    use aira_object::{AiraRef, ContentHash};

    fn registry() -> aira_schema::SchemaRegistry {
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap()
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn envelope_schema_valid_and_unsigned_rejected() {
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = registry();
        let valid =
            std::fs::read_to_string(root.join("fixtures/valid/protocol/envelope.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&valid).unwrap();
        reg.validate("aira:schema:protocol:envelope:0.1", &v)
            .unwrap();

        let env: ProtocolEnvelope = serde_json::from_value(v).unwrap();
        assert_eq!(env.protocol_id, ProtocolId::Event);
        // Schema fixtures are not live crypto objects (same rule as CSU manifest fixtures).
        let live = env.attach_canonical_signature().unwrap();
        live.validate_signature().unwrap();

        let bad =
            std::fs::read_to_string(root.join("fixtures/invalid/protocol/envelope-unsigned.json"))
                .unwrap();
        let bad_v: serde_json::Value = serde_json::from_str(&bad).unwrap();
        assert!(reg
            .validate("aira:schema:protocol:envelope:0.1", &bad_v)
            .is_err());
    }

    #[test]
    fn response_and_identity_schema_valid() {
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = registry();
        for (schema, path) in [
            (
                "aira:schema:protocol:response:0.1",
                "fixtures/valid/protocol/response.json",
            ),
            (
                "aira:schema:identity:identity-descriptor:0.1",
                "fixtures/valid/identity/identity-descriptor.json",
            ),
        ] {
            let text = std::fs::read_to_string(root.join(path)).unwrap();
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            reg.validate(schema, &v).unwrap();
        }
        let id = IdentityDescriptor::local_user(
            "aira:identity:local-user",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        assert!(!id.public_keys.is_empty());
        assert!(!id.signature.signature_value.is_empty());
        let v = serde_json::to_value(&id).unwrap();
        reg.validate("aira:schema:identity:identity-descriptor:0.1", &v)
            .unwrap();
    }

    #[test]
    fn event_protocol_publish_idempotent_and_unsupported_version() {
        let mut ep = EventProtocolAdapter::new();
        let ev = make_event(
            "aira:event:ep_test1",
            EventType::ProblemSubmitted,
            vec![AiraRef::parse("aira:problem:p1").unwrap()],
            vec![],
            vec![],
            Some("hello".into()),
        );
        let (env, resp) = ep.publish_event(ev.clone(), EP_VERSION).unwrap();
        assert_eq!(env.protocol_id, ProtocolId::Event);
        assert_eq!(resp.status, ProtocolStatus::Accepted);
        assert_eq!(ep.events().len(), 1);

        let (_env2, resp2) = ep.publish_event(ev.clone(), EP_VERSION).unwrap();
        assert_eq!(resp2.status, ProtocolStatus::Accepted);
        assert_eq!(ep.events().len(), 1, "duplicate must be idempotent");

        let (_env3, resp3) = ep.publish_event(ev, "9.9").unwrap();
        assert_eq!(resp3.status, ProtocolStatus::UnsupportedVersion);
    }

    #[test]
    fn event_protocol_rejects_equivocation() {
        let mut ep = EventProtocolAdapter::new();
        let ev_a = make_event(
            "aira:event:ep_equiv",
            EventType::ProblemSubmitted,
            vec![AiraRef::parse("aira:problem:p1").unwrap()],
            vec![],
            vec![],
            Some("a".into()),
        );
        let ev_b = make_event(
            "aira:event:ep_equiv",
            EventType::ProblemSubmitted,
            vec![AiraRef::parse("aira:problem:p1").unwrap()],
            vec![],
            vec![],
            Some("b".into()),
        );
        let (_env, resp) = ep.publish_event(ev_a, EP_VERSION).unwrap();
        assert_eq!(resp.status, ProtocolStatus::Accepted);
        let (_env2, resp2) = ep.publish_event(ev_b, EP_VERSION).unwrap();
        assert_eq!(resp2.status, ProtocolStatus::Equivocation);
        assert_eq!(ep.events().len(), 1);
    }

    #[test]
    fn artifact_protocol_publish_resolve_and_hash_check() {
        let dir = tempfile::tempdir().unwrap();
        let mut ap = ArtifactProtocolAdapter::open(dir.path()).unwrap();
        let payload = b"protocol-artifact";
        let desc = make_artifact(
            "aira:artifact:ap_test1",
            ArtifactType::EvidenceArtifact,
            payload,
            vec![],
        );
        let out = ap.publish(desc.clone(), payload, AP_VERSION).unwrap();
        assert_eq!(out.response.status, ProtocolStatus::Accepted);
        assert!(out.published.is_some());

        let got = ap.resolve(&desc.artifact_id, AP_VERSION).unwrap();
        assert_eq!(got.response.status, ProtocolStatus::Accepted);
        let (_d, bytes) = got.resolved.unwrap();
        assert_eq!(bytes, payload);

        let bad_hash = {
            let mut d = desc.clone();
            d.content_hash = ContentHash::parse(
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            )
            .unwrap();
            d
        };
        let bad = ap.publish(bad_hash, payload, AP_VERSION).unwrap();
        assert_eq!(bad.response.status, ProtocolStatus::InvalidArtifact);

        let unsup = ap.publish(desc, payload, "2.0").unwrap();
        assert_eq!(unsup.response.status, ProtocolStatus::UnsupportedVersion);
    }

    #[test]
    fn discovery_returns_capability_not_node() {
        let mut reg = DiscoveryRegistry::new();
        let cap = DiscoveryRegistry::local_capability(
            "aira:capability:math.eval.safe",
            "math.eval.safe",
            "aira:csu:execution.basic",
        )
        .unwrap();
        reg.register(cap).unwrap();
        let hits = reg.query("math.eval.safe");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].provider_csu.as_str(), "aira:csu:execution.basic");
        assert!(!hits[0].capability.capability_id.as_str().contains(":node:"));
        let v = serde_json::to_value(&hits[0].capability).unwrap();
        assert!(v.get("provider_csu").is_some());
        assert!(v.get("node_id").is_none());
        assert_eq!(reg.list_all().len(), 1);
    }

    #[test]
    fn envelope_rejects_local_test_domain_fallback() {
        let hash = ContentHash::sha256_bytes(b"envelope-body");
        let env = ProtocolEnvelope {
            protocol_id: ProtocolId::Event,
            protocol_version: EP_VERSION.into(),
            message_type: "EventPublish".into(),
            message_id: AiraRef::parse("aira:message:ep-domain-reject").unwrap(),
            correlation_id: None,
            causal_refs: vec![],
            issuer_identity: local_identity(),
            target_scope: ScopeDescriptor::local("event-protocol"),
            policy_refs: vec![AiraRef::parse("aira:policy:default").unwrap()],
            payload_hash: hash.clone(),
            payload_ref: None,
            created_at: mvp_timestamp(),
            expires_at: None,
            signature: local_signature(),
        };
        assert!(env.validate_signature().is_err());

        let signed = env.attach_canonical_signature().unwrap();
        signed.validate_signature().unwrap();
    }

    #[test]
    fn identity_rejects_local_test_domain_signature() {
        let mut id = IdentityDescriptor::local_user(
            "aira:identity:local-user",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        id.signature = local_signature();
        assert!(
            aira_object::verify_ed25519(&id.signature, id.identity_id.as_str().as_bytes()).is_err()
        );
        assert!(
            aira_object::verify_ed25519(&id.signature, aira_object::LOCAL_TEST_DOMAIN_MSG).is_ok()
        );
    }
}
