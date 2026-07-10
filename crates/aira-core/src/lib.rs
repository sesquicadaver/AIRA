//! AIRA core runtime orchestration (Issue Set Epic 3 / #25–#26).
//!
//! Provides immutable Object Store (memory + SQLite) and invariant errors.
//! Core does **not** contain domain/ML/GPU/scheduling logic.

mod error;
mod sqlite;
mod store;

pub use error::{CoreError, InvariantViolation};
pub use sqlite::SqliteObjectStore;
pub use store::{MemoryObjectStore, ObjectStore};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{AiraRef, ObjectDescriptor};

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

        // reopen
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
}
