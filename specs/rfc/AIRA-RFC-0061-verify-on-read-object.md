# AIRA-RFC-0061 — Object verify-on-read

## 1. Summary

Phase F `#112`: `ObjectStore::open` and `get_by_object_id` re-verify canonical object descriptor signatures on read. Tampered `descriptor_json` in SQLite (or in-memory store) returns `CoreError::InvalidSignature`.

## 5. Non-Goals

Sqlite schema migration; new descriptor fields; artifact verify-on-read (#113).

## 15. Tests

`cargo test -p aira-core`
`cargo test -p aira-conformance --lib`
`cargo run -p aira-cli -- conformance run --profile C0` (case `c0.object.verify_on_read`)
