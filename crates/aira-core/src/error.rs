//! Core errors and invariant violations.

use thiserror::Error;

use aira_object::AiraRef;

/// Invariant violation candidates emitted by Core (Book I spirit).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvariantViolation {
    #[error("object immutability violated for {object_id}")]
    ObjectImmutability { object_id: AiraRef },
}

/// Core runtime errors.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Invariant(#[from] InvariantViolation),
    #[error("object not found: {0}")]
    NotFound(AiraRef),
    #[error("duplicate object id: {object_id}")]
    DuplicateObject { object_id: AiraRef },
    #[error("storage error: {0}")]
    Storage(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
