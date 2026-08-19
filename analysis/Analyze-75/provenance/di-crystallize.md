# DI crystallize — Analyze-75 / QUEUE #40

## In
1. Event production signatures: Canonical JSON without top-level `signature` → SHA-256 → Ed25519 over `hash.as_str()` bytes (`sign_canonical_descriptor` / tenant `descriptor_signing_message`).
2. Event admission: `verify_canonical` only. No `payload_hash`-only verify and no `LOCAL_TEST_DOMAIN_MSG` fallback on the event path (`log.rs`, `invariants.rs`).
3. Mutation tests for `event_type`, `causal_refs`, `object_refs`, `artifact_refs`, `payload_hash`.
4. Re-sign after mutating `payload_ref` in secret-in-event tests (canonical covers that field).

## Out
Artifact/Object/CSU descriptor wiring; leftover envelope/identity fallback (#44).
