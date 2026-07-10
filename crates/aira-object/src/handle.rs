//! Opaque object handle — CSU must not learn storage paths from it.

use crate::types::AiraRef;

/// Opaque handle to a stored Core Object.
///
/// Debug output intentionally omits any filesystem / SQL path. The only
/// public logical identifier is [`Handle::object_ref`].
#[derive(Clone, PartialEq, Eq)]
pub struct Handle {
    object_ref: AiraRef,
    /// Internal store token (row id / generation). Not a path.
    storage_token: u64,
}

impl Handle {
    /// Construct a handle from a logical object ref and internal token.
    pub fn new(object_ref: AiraRef, storage_token: u64) -> Self {
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
    pub fn storage_token(&self) -> u64 {
        self.storage_token
    }

    /// Test-only alias.
    #[doc(hidden)]
    pub fn storage_token_for_tests(&self) -> u64 {
        self.storage_token()
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
