# DI crystallize — Analyze-82 / QUEUE #47

## In
1. Replace `crypto.rs` with `crypto/mod.rs` and four modules: error, keyring, trust_store, rotation.
2. Keep public `aira_object::crypto` / crate re-exports identical.
3. Keep `#[cfg(test)]` suite in `crypto/mod.rs`.
4. CRL/revoke/rekey stay on `TrustStore` in `trust_store.rs` (no separate `tenant_signing`; that is `tenant.rs` / #48).

## Out
`tenant.rs` file split; CLI; HTTP; new crypto behavior.
