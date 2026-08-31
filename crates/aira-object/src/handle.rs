//! Opaque object handle — CSU must not learn storage paths from it.

use crate::types::AiraRef;

/// Opaque handle to a stored Core Object.
///
/// Debug output intentionally omits any filesystem / SQL path. The only
/// public logical identifier is [`Handle::object_ref`].
///
/// Construction and storage-token access are **not** on this type's public
/// API (`Handle::new` / `Handle::storage_token` are `pub(crate)`). Store
/// implementations in `aira-core` use `object_store_access` (feature `store-backend`).
#[derive(Clone, PartialEq, Eq)]
pub struct Handle {
    object_ref: AiraRef,
    /// Internal store token (row id / generation). Not a path.
    storage_token: u64,
}

impl Handle {
    /// Construct a handle from a logical object ref and internal token.
    pub(crate) fn new(object_ref: AiraRef, storage_token: u64) -> Self {
        Self {
            object_ref,
            storage_token,
        }
    }

    /// Logical object reference (safe to expose to CSU).
    pub fn object_ref(&self) -> &AiraRef {
        &self.object_ref
    }

    /// Internal store token (row id / generation). **Not** a filesystem path.
    pub(crate) fn storage_token(&self) -> u64 {
        self.storage_token
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("object_ref", &self.object_ref.as_str())
            .field("token", &"<opaque>")
            .finish()
    }
}

/// Mint and token access for `aira-core` ObjectStore implementations.
///
/// Gated by Cargo feature `store-backend` (enabled only on `aira-core`).
/// This is **not** a CSU API. CSUs receive [`Handle`] only from `ObjectStore::create`
/// and may read [`Handle::object_ref`]. Forged mint + `open` is still bind-checked
/// (`object_id == handle.object_ref`) in Core.
#[cfg(feature = "store-backend")]
pub mod object_store_access {
    use super::{AiraRef, Handle};

    /// Mint a handle for a store that just persisted `object_ref` at `storage_token`.
    pub fn mint(object_ref: AiraRef, storage_token: u64) -> Handle {
        Handle::new(object_ref, storage_token)
    }

    /// Read the store token. For ObjectStore backends only.
    pub fn storage_token(handle: &Handle) -> u64 {
        handle.storage_token()
    }
}
