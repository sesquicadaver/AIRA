# Provenance — Minimal Trust CRL

**Decision:** Durable `revoked[]` in `trust.json`; `revoke` ≠ `remove`; upsert blocked by `RevokedKey`.

**Why:** Analyze-24 unload alone allowed silent re-trust after remove.

**Non-goals:** Unrevoke admin, rotation supersedes chain, dual-key verify window.
