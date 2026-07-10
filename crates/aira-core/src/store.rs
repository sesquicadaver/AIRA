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

impl ObjectStore for MemoryObjectStore {
    fn create(&mut self, descriptor: ObjectDescriptor) -> Result<Handle, CoreError> {
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
        self.by_token
            .get(&handle.storage_token())
            .cloned()
            .ok_or_else(|| CoreError::NotFound(handle.object_ref().clone()))
    }

    fn get_by_object_id(&self, object_id: &AiraRef) -> Result<Option<ObjectDescriptor>, CoreError> {
        Ok(self
            .by_id
            .get(object_id.as_str())
            .and_then(|t| self.by_token.get(t))
            .cloned())
    }
}
