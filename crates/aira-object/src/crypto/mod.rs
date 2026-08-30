//! Ed25519 helpers + process keyring (Alpha.2 / Analyze-21).
//!
//! Mechanical split (Analyze-82 / QUEUE #47; tests → `smoke_tests.rs` QUEUE #127).

mod error;
mod keyring;
mod rotation;
mod trust_store;

pub(crate) use error::parse_rfc3339;
pub use error::{utc_now_rfc3339, CryptoError, LOCAL_TEST_DOMAIN_MSG, LOCAL_TEST_KEY_REF};
pub use keyring::{
    active_identity, active_signature, is_cryptographic_signature, local_test_public_key_hex,
    local_test_signature, local_test_signing_key, local_test_verifying_key, primary_signer,
    process_keyring_snapshot, register_keyring, register_node_identity, reset_primary_signer,
    set_primary_signer, sign_with_key, signature_for, unregister_verifying, verify_ed25519,
    Keyring,
};
pub(crate) use rotation::should_retain_archived;
pub use rotation::{
    list_node_secret_backups, prune_node_secret_backups, rotate_node_signing_secret,
    NodeSecretBackupInfo, NodeSecretPruneReport, NODE_SECRET_BACKUP_FILE,
    NODE_SECRET_BACKUP_META_FILE,
};
pub use trust_store::{
    ensure_trust_defaults, register_trust_store, sync_trust_verifiers, RevokedEntry, TrustEntry,
    TrustStore,
};

#[cfg(test)]
mod smoke_tests;
