//! AIRA basic execution-basic CSU skeleton
//!
//! Skeleton crate for AIRA MVP bootstrap (Issue Set Epic 0). No domain logic yet.

/// Crate version string for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }
}
