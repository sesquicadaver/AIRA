# Analyze-50 — Remote same-id dual-key TrustStore

**Scope:** `TrustEntry` previous_* + `TrustStore::rekey` with grace; `apply_trust_delta` Rekey uses it; `to_keyring_at` dual-verify. No mTLS. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## RALPLAN-DR

### Principles
1. Same-id grace mirrors local node previous_* (A-37)
2. Wire `grace_until` on rekey is authoritative
3. No grace → immediate cutover
4. Old trust.json without previous_* loads cleanly
5. mTLS stays #15

### Chosen
**A.** TrustEntry previous_* + rekey() — not mTLS.

### Acceptance
- [x] rekey+grace → both keys verify same id until cutoff
- [x] rekey without grace → only new key
- [x] apply_trust_delta Rekey uses store.rekey
- [x] docs + QUEUE #16 DONE; tests + APPROVE/CLEAR

### Delivered
- `TrustEntry::{previous_public_key_hex, previous_grace_until}`
- `TrustStore::rekey`
- Peer notify+grace integration test

### Out
mTLS; multi-previous history; x25519 notify.
