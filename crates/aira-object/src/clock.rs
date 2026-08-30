//! Runtime clock (QUEUE #193).
//!
//! Operational emits use [`now`]. Tests may install [`FixedClock`]. Protocol
//! fixtures may still use a literal MVP timestamp independently.

use std::sync::{Arc, OnceLock, RwLock};

use crate::crypto::utc_now_rfc3339;
use crate::types::{Timestamp, TypeError};

/// Historical fixture timestamp (not the runtime default).
pub const MVP_FIXED_TIMESTAMP: &str = "2026-07-10T12:00:00Z";

/// Source of `created_at` for operational descriptors.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// Wall-clock UTC via [`utc_now_rfc3339`].
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let raw = utc_now_rfc3339().expect("system utc rfc3339");
        Timestamp::parse(raw).expect("clock rfc3339")
    }
}

/// Deterministic clock for tests.
pub struct FixedClock {
    ts: Timestamp,
}

impl FixedClock {
    pub fn parse(s: &str) -> Result<Self, TypeError> {
        Ok(Self {
            ts: Timestamp::parse(s)?,
        })
    }

    pub fn mvp() -> Self {
        Self {
            ts: Timestamp::parse(MVP_FIXED_TIMESTAMP).expect("mvp ts"),
        }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.ts.clone()
    }
}

fn slot() -> &'static RwLock<Arc<dyn Clock>> {
    static SLOT: OnceLock<RwLock<Arc<dyn Clock>>> = OnceLock::new();
    SLOT.get_or_init(|| RwLock::new(Arc::new(SystemClock)))
}

/// Install the process clock (tests).
pub fn set_clock(clock: Arc<dyn Clock>) {
    let mut g = slot().write().unwrap_or_else(|e| e.into_inner());
    *g = clock;
}

/// Restore [`SystemClock`].
pub fn reset_clock() {
    set_clock(Arc::new(SystemClock));
}

/// Current process clock time.
pub fn now() -> Timestamp {
    slot().read().unwrap_or_else(|e| e.into_inner()).now()
}

/// Unix seconds for a stored RFC3339 timestamp.
pub fn unix_seconds(ts: &Timestamp) -> Result<i64, crate::CryptoError> {
    crate::crypto::parse_rfc3339(ts.as_str()).map(|dt| dt.unix_timestamp())
}

/// Unix seconds for a raw RFC3339 string (`expires_at`).
pub fn unix_seconds_str(s: &str) -> Result<i64, crate::CryptoError> {
    crate::crypto::parse_rfc3339(s).map(|dt| dt.unix_timestamp())
}
