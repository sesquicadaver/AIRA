# AIRA-RFC-0084 — Handle integrity (bind + non-public mint)

## 1. Summary

Phase I `#186`: `Handle::new` and `Handle::storage_token` are crate-private. Store backends mint via `aira_object::object_store_access`. `ObjectStore::open` binds `descriptor.object_id == handle.object_ref()` before verify-on-read. Forged, cross-object, and cross-store opens fail.

## 5. Non-Goals

Semantic Verification (`#187`); PolicyGate in invoke (`#188`); durable reuse (`#189`); changing C0 case ids; CSU-visible `object_ref`.

## 10. Contract

```text
Handle::new / Handle::storage_token — not public methods
object_store_access::{mint, storage_token} — store crate only (not CSU API)
open bind — HandleBindMismatch when token row ≠ claimed object_ref
```

## 15. Tests

```text
cargo test -p aira-object handle_
cargo test -p aira-core handle_
cargo test -p aira-conformance c0
```
