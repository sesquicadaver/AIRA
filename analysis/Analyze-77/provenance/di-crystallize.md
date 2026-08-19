# DI crystallize — Analyze-77 / QUEUE #42

## In
1. Object production signatures: canonical JSON without top-level `signature`.
2. Object store `create` (memory + SQLite) verifies `verify_canonical` only.
3. Mutation tests for `object_type`, `policy_refs`, `provenance_refs`, `content_hash`, `object_id`.
4. Plane problem objects attach canonical before create.

## Out
CSU manifests (#43); leftover envelope fallback (#44).
