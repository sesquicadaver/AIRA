//! AIRA event runtime (Issue Set Epic 4 / #30–#32).
//!
//! Append-only event log with local subscriptions. No global total order.

mod descriptor;
mod log;

pub use descriptor::{EventDescriptor, EventType};
pub use log::{
    payload_contains_secret, EventError, EventLog, EventSink, MemoryEventLog, SubscriptionId,
};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{AiraRef, ContentHash, Timestamp};
    use std::sync::{Arc, Mutex};

    fn sample_event(event_id: &str, event_type: EventType) -> EventDescriptor {
        let payload_hash = ContentHash::parse(
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap();
        EventDescriptor {
            event_id: AiraRef::parse(event_id).unwrap(),
            event_type,
            schema_version: "0.1".into(),
            producer_identity: AiraRef::parse("aira:identity:local-test").unwrap(),
            causal_refs: vec![],
            object_refs: vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            artifact_refs: vec![AiraRef::parse(
                "aira:artifact:sha256_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            )
            .unwrap()],
            policy_refs: vec![],
            payload_hash: payload_hash.clone(),
            payload_ref: None,
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
            signature: aira_object::local_test_signature(payload_hash.as_str().as_bytes()),
        }
        .attach_canonical_signature()
        .expect("canonical sample")
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn event_schema_valid() {
        let e = sample_event("aira:event:01E1", EventType::ProblemSubmitted);
        let v = serde_json::to_value(&e).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:event:event-descriptor:0.1", &v)
            .unwrap();
    }

    #[test]
    fn append_only_and_query() {
        let mut log = MemoryEventLog::new();
        let e = sample_event("aira:event:01E1", EventType::ProblemSubmitted);
        log.append(e.clone()).unwrap();
        let err = log.mutate(&e.event_id).unwrap_err();
        assert!(matches!(err, EventError::Immutable { .. }));

        let by_obj =
            log.query_by_object_ref(&AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap());
        assert_eq!(by_obj.len(), 1);
        let by_art = log.query_by_artifact_ref(&AiraRef::parse(
            "aira:artifact:sha256_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap());
        assert_eq!(by_art.len(), 1);
    }

    #[test]
    fn subscriptions_idempotent_delivery() {
        let mut log = MemoryEventLog::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let r2 = received.clone();
        log.subscribe(EventType::ProblemSubmitted, move |ev| {
            r2.lock().unwrap().push(ev.event_id.as_str().to_string());
        });

        let e = sample_event("aira:event:01E1", EventType::ProblemSubmitted);
        log.append(e.clone()).unwrap();
        // duplicate append is idempotent (no second delivery)
        log.append(e).unwrap();
        assert_eq!(received.lock().unwrap().len(), 1);
    }

    #[test]
    fn secret_material_rejected_in_payload_ref() {
        let mut log = MemoryEventLog::new();
        let mut e = sample_event("aira:event:sec1", EventType::CustomEvent);
        e.payload_ref = Some("password=hunter2".into());
        e = e
            .attach_canonical_signature()
            .expect("re-sign after payload_ref");
        let err = log.append(e).unwrap_err();
        assert!(matches!(err, EventError::SecretMaterial));
        assert!(payload_contains_secret(Some("BEGIN PRIVATE KEY-----")));
        assert!(!payload_contains_secret(Some("Calculate 2 + 2")));
    }

    #[test]
    fn canonical_verify_fails_when_event_type_or_causal_refs_change() {
        let e = sample_event("aira:event:mut1", EventType::ProblemSubmitted);
        e.verify_canonical().unwrap();

        let mut t = e.clone();
        t.event_type = EventType::ResultPublished;
        assert!(t.verify_canonical().is_err());

        let mut c = e.clone();
        c.causal_refs = vec![AiraRef::parse("aira:event:01E0").unwrap()];
        assert!(c.verify_canonical().is_err());

        let mut o = e.clone();
        o.object_refs = vec![AiraRef::parse("aira:problem:MUTATED").unwrap()];
        assert!(o.verify_canonical().is_err());

        let mut a = e.clone();
        a.artifact_refs = vec![AiraRef::parse(
            "aira:artifact:sha256_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap()];
        assert!(a.verify_canonical().is_err());

        let mut h = e;
        h.payload_hash = ContentHash::parse(
            "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        )
        .unwrap();
        assert!(h.verify_canonical().is_err());
    }
}
