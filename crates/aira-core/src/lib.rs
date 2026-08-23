//! AIRA core runtime orchestration (Issue Set Epic 3–4).
//!
//! Provides immutable Object Store, Invariant Checker, and invariant errors.
//! Core does **not** contain domain/ML/GPU/scheduling logic.

mod error;
mod invariants;
mod sqlite;
mod store;

pub use error::{CoreError, InvariantViolation};
pub use invariants::InvariantChecker;
pub use sqlite::SqliteObjectStore;
pub use store::{MemoryObjectStore, ObjectStore};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_event::{EventType, MemoryEventLog};
    use aira_object::Timestamp;
    use aira_object::{AiraRef, ObjectDescriptor, Signature};
    use aira_policy::{PolicyGate, PolicyQuery};

    fn sig() -> Signature {
        aira_object::local_test_signature(aira_object::LOCAL_TEST_DOMAIN_MSG)
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn memory_create_open() {
        let mut store = MemoryObjectStore::new();
        let desc = ObjectDescriptor::example_problem();
        let handle = store.create(desc.clone()).unwrap();
        assert_eq!(handle.object_ref(), &desc.object_id);
        let loaded = store.open(&handle).unwrap();
        assert_eq!(loaded, desc);
    }

    #[test]
    fn memory_rejects_in_place_mutation() {
        let mut store = MemoryObjectStore::new();
        let desc = ObjectDescriptor::example_problem();
        let handle = store.create(desc.clone()).unwrap();
        let mut mutated = desc.clone();
        mutated.schema_version = "0.2".into();
        let err = store.replace_in_place(&handle, mutated).unwrap_err();
        match err {
            CoreError::Invariant(InvariantViolation::ObjectImmutability { .. }) => {}
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn sqlite_persist_and_lookup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("objects.db");
        let mut store = SqliteObjectStore::open(&path).unwrap();
        let desc = ObjectDescriptor::example_problem();
        let handle = store.create(desc.clone()).unwrap();
        let loaded = store.open(&handle).unwrap();
        assert_eq!(loaded.object_id, desc.object_id);

        let by_id = store
            .get_by_object_id(&AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(by_id, desc);

        drop(store);
        let store2 = SqliteObjectStore::open(&path).unwrap();
        let again = store2.get_by_object_id(&desc.object_id).unwrap().unwrap();
        assert_eq!(again, desc);
    }

    #[test]
    fn sqlite_duplicate_insert_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("objects.db");
        let mut store = SqliteObjectStore::open(&path).unwrap();
        let desc = ObjectDescriptor::example_problem();
        store.create(desc.clone()).unwrap();
        let err = store.create(desc).unwrap_err();
        match err {
            CoreError::DuplicateObject { .. } => {}
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn create_rejects_unsigned_and_mutated_object_signature() {
        let mut store = MemoryObjectStore::new();
        let mut unsigned = ObjectDescriptor::example_problem();
        unsigned.signature.signature_value.clear();
        assert!(matches!(
            store.create(unsigned),
            Err(CoreError::Unsigned(_))
        ));

        let mut bad = ObjectDescriptor::example_problem();
        bad.schema_version = "0.2".into();
        assert!(matches!(
            store.create(bad),
            Err(CoreError::InvalidSignature(_))
        ));
    }

    #[test]
    fn open_and_get_by_id_reject_tampered_descriptor() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("objects.db");
        let mut store = SqliteObjectStore::open(&path).unwrap();
        let desc = ObjectDescriptor::example_problem();
        let object_id = desc.object_id.clone();
        let handle = store.create(desc).unwrap();

        let conn = rusqlite::Connection::open(&path).unwrap();
        let mut tampered = ObjectDescriptor::example_problem();
        tampered.schema_version = "0.2".into();
        let tampered_json = serde_json::to_string(&tampered).unwrap();
        conn.execute(
            "UPDATE objects SET descriptor_json = ?1 WHERE object_id = ?2",
            rusqlite::params![tampered_json, object_id.as_str()],
        )
        .unwrap();

        let reopened = SqliteObjectStore::open(&path).unwrap();
        assert!(matches!(
            reopened.open(&handle),
            Err(CoreError::InvalidSignature(_))
        ));
        assert!(matches!(
            reopened.get_by_object_id(&object_id),
            Err(CoreError::InvalidSignature(_))
        ));
    }

    #[test]
    fn invariant_checker_emits_event_on_policy_deny() {
        let mut log = MemoryEventLog::new();
        let mut checker =
            InvariantChecker::new(AiraRef::parse("aira:identity:local-test").unwrap(), sig());
        let mut gate = PolicyGate::new(sig());
        let q = PolicyQuery {
            subject: AiraRef::parse("aira:csu:ctx.basic").unwrap(),
            csu_ref: None,
            action: "secret_exfiltrate".into(),
            object_refs: vec![],
            artifact_refs: vec![],
            context_refs: vec![],
            evidence_refs: vec![],
            requested_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
        };
        let err = checker
            .check_policy_before_action(&mut gate, q, &mut log)
            .unwrap_err();
        assert!(matches!(
            err,
            CoreError::Invariant(InvariantViolation::PolicyDenied { .. })
        ));
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::InvariantViolation));
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::PolicyEvaluated));
    }
}
