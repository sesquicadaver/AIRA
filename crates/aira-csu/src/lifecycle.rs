//! CSU lifecycle state machine (Issue #37).

use serde::{Deserialize, Serialize};

use crate::error::CsuError;

/// Canonical CSU lifecycle states (Book I §14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CsuLifecycleState {
    Discovered,
    Registered,
    Verified,
    Active,
    Suspended,
    Revoked,
    Archived,
}

impl CsuLifecycleState {
    /// Whether a transition `self → to` is allowed.
    pub fn can_transition_to(self, to: Self) -> bool {
        use CsuLifecycleState::*;
        matches!(
            (self, to),
            (Discovered, Registered)
                | (Registered, Verified)
                | (Verified, Active)
                | (Active, Suspended)
                | (Suspended, Active)
                | (Suspended, Revoked)
                | (Active, Revoked)
                | (Revoked, Archived)
                | (Suspended, Archived)
                | (Verified, Suspended)
        )
    }

    /// Transition or error.
    pub fn transition(self, to: Self) -> Result<Self, CsuError> {
        if self.can_transition_to(to) {
            Ok(to)
        } else {
            Err(CsuError::InvalidTransition { from: self, to })
        }
    }
}
