# DI crystallize — Analyze-74 / QUEUE #39

## In
1. New `aira-object` module `canonical.rs` (not a split of `crypto.rs`).
2. Canonical JSON: sorted keys, compact UTF-8, SHA-256, sign/verify `ContentHash.as_str()` bytes.
3. Strip only top-level `signature`.
4. Tests: order/whitespace, strip, mutation, roundtrip, independence from payload_hash-only verify.

## Out
Switching InvariantChecker / event log / artifact / object / CSU; removing LOCAL_TEST fallback.
