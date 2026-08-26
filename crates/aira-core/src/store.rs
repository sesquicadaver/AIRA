//! Immutable Object Store trait and in-memory implementation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use aira_object::{AiraRef, Handle, ObjectDescriptor};

use crate::error::{CoreError, InvariantViolation};

/// Immutable object store API (Issue #25).
pub trait ObjectStore {
    /// Persist a new object descriptor; returns an opaque handle.
    fn create(&mut self, descriptor: ObjectDescriptor) -> Result<Handle, CoreError>;

    /// Open descriptor by handle.
    fn open(&self, handle: &Handle) -> Result<ObjectDescriptor, CoreError>;

    /// Lookup by logical object id.
    fn get_by_object_id(&self, object_id: &AiraRef) -> Result<Option<ObjectDescriptor>, CoreError>;

    /// Attempt in-place replace — **must** fail with ObjectImmutability.
    fn replace_in_place(
        &mut self,
        handle: &Handle,
        _new_descriptor: ObjectDescriptor,
    ) -> Result<(), CoreError> {
        Err(CoreError::Invariant(
            InvariantViolation::ObjectImmutability {
                object_id: handle.object_ref().clone(),
            },
        ))
    }
}

/// In-memory Object Store for tests and local prototypes.
#[derive(Debug, Default)]
pub struct MemoryObjectStore {
    by_token: HashMap<u64, ObjectDescriptor>,
    by_id: HashMap<String, u64>,
    next_token: AtomicU64,
}

impl MemoryObjectStore {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Admit an object descriptor: cryptographic signature over canonical JSON.
pub(crate) fn admit_object(descriptor: &ObjectDescriptor) -> Result<(), CoreError> {
    if descriptor.signature.signature_value.trim().is_empty() {
        return Err(CoreError::Unsigned(descriptor.object_id.clone()));
    }
    match descriptor.verify_canonical() {
        Ok(()) => Ok(()),
        Err(aira_object::CryptoError::MissingOrLegacy) => {
            Err(CoreError::Unsigned(descriptor.object_id.clone()))
        }
        Err(_) => Err(CoreError::InvalidSignature(descriptor.object_id.clone())),
    }
}

/// Verify-on-read: re-check canonical signature before returning a stored descriptor.
pub(crate) fn verify_stored_descriptor(
    descriptor: ObjectDescriptor,
) -> Result<ObjectDescriptor, CoreError> {
    admit_object(&descriptor)?;
    Ok(descriptor)
}

impl ObjectStore for MemoryObjectStore {
    fn create(&mut self, descriptor: ObjectDescriptor) -> Result<Handle, CoreError> {
        admit_object(&descriptor)?;
        let id_key = descriptor.object_id.as_str().to_string();
        if self.by_id.contains_key(&id_key) {
            return Err(CoreError::DuplicateObject {
                object_id: descriptor.object_id,
            });
        }
        let token = self.next_token.fetch_add(1, Ordering::Relaxed) + 1;
        let handle = Handle::new(descriptor.object_id.clone(), token);
        self.by_id.insert(id_key, token);
        self.by_token.insert(token, descriptor);
        Ok(handle)
    }

    fn open(&self, handle: &Handle) -> Result<ObjectDescriptor, CoreError> {
        let descriptor = self
            .by_token
            .get(&handle.storage_token())
            .cloned()
            .ok_or_else(|| CoreError::NotFound(handle.object_ref().clone()))?;
        verify_stored_descriptor(descriptor)
    }

    fn get_by_object_id(&self, object_id: &AiraRef) -> Result<Option<ObjectDescriptor>, CoreError> {
        let descriptor = self
            .by_id
            .get(object_id.as_str())
            .and_then(|t| self.by_token.get(t))
            .cloned();
        match descriptor {
            None => Ok(None),
            Some(d) => Ok(Some(verify_stored_descriptor(d)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use aira_object::ObjectDescriptor;

    use super::*;

    #[test]
    fn memory_open_and_get_reject_tampered_descriptor() {
        let mut store = MemoryObjectStore::new();
        let desc = ObjectDescriptor::example_problem();
        let object_id = desc.object_id.clone();
        let handle = store.create(desc).unwrap();
        let token = handle.storage_token();
        let mut tampered = ObjectDescriptor::example_problem();
        tampered.schema_version = "0.2".into();
        store.by_token.insert(token, tampered);

        assert!(matches!(
            store.open(&handle),
            Err(CoreError::InvalidSignature(_))
        ));
        assert!(matches!(
            store.get_by_object_id(&object_id),
            Err(CoreError::InvalidSignature(_))
        ));
    }

    #[test]
    fn create_rejects_cross_identity_key_ref() {
        let mut store = MemoryObjectStore::new();
        let mut desc = ObjectDescriptor::example_problem();
        desc.signature.key_ref = aira_object::AiraRef::parse("aira:identity:other-signer").unwrap();
        assert!(matches!(
            store.create(desc).unwrap_err(),
            CoreError::InvalidSignature(_)
        ));
    }
}
