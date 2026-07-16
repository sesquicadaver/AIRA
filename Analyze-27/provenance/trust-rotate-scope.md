# Provenance — Minimal Trust Rotate

**Decision:** Atomic `rotate(old, new, pubkey)` = revoke old with `superseded_by` + upsert new with `supersedes`. No dual-key verify window.

**Why:** Peer key replacement without manual revoke+add race; metadata for audit/chain.

**Non-goals:** Grace TTL dual-key verify; node `local.ed25519` rotate; graph walk of supersedes chains.
