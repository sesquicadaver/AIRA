# AIRA-RFC-0093 — Run nonce UUIDv7

## 1. Summary

Phase I `#195`: each local submit allocates a UUIDv7 run nonce instead of parse/write on a shared `run-counter` file. Concurrent processes no longer collide on artifact/event ids.

## 5. Non-Goals

Instance-scoped crypto (`#196`); rewriting historical ids already on disk; requiring a lock file.

## 10. Contract

```text
alloc_run_nonce() → 32-char UUIDv7 hex (no hyphens)
LocalSession::open / submit_problem → that nonce into OperationalPlane + CSU with_run_nonce
legacy `.aira/run-counter` is ignored (not read, not written)
ids: aira:problem:flow{nonce}_{seq} (same pattern; nonce is hex, not an integer)
```

## 15. Tests

```text
cargo test -p aira-flow --lib alloc_run_nonce_concurrent_is_unique
cargo test -p aira-flow --lib two_submits_allocate_distinct_problem_ids
```
