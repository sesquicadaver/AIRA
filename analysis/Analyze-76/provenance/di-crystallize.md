# DI crystallize — Analyze-76 / QUEUE #41

## In
1. Artifact production signatures: canonical JSON without top-level `signature`.
2. CAS `publish` verifies `verify_canonical` only (no `content_hash`-only message).
3. Mutation tests for type, provenance, policy_refs, content_hash.
4. Re-sign after mutating `policy_refs` in private-artifact tests.

## Out
Object/CSU; Event (already #40); CAS directory layout.
