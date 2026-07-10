//! CSU runtime errors.

use thiserror::Error;

use aira_object::AiraRef;

/// Errors from CSU registry / runtime.
#[derive(Debug, Error)]
pub enum CsuError {
    #[error("manifest validation failed: {0}")]
    ManifestInvalid(String),
    #[error("unsigned CSU manifest: {0}")]
    UnsignedManifest(AiraRef),
    #[error("unsupported ABI version: {0}")]
    UnsupportedAbi(String),
    #[error("CSU not found: {0}")]
    NotFound(AiraRef),
    #[error("invalid lifecycle transition: {from:?} → {to:?}")]
    InvalidTransition {
        from: crate::lifecycle::CsuLifecycleState,
        to: crate::lifecycle::CsuLifecycleState,
    },
    #[error("CSU not active: {0}")]
    NotActive(AiraRef),
    #[error("isolation violation: {0}")]
    Isolation(String),
    #[error("dispatch failure: {0}")]
    Dispatch(String),
    #[error("storage error: {0}")]
    Storage(String),
}
