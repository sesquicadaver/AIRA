# Analyze-37 — Dual-key node grace

**Scope:** Multi-pubkey `Keyring` for the same node `key_ref`; `identity rotate --until` keeps old signatures verifiable until grace ends. No peer notify.

**Status:** DONE — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Same `identity_id` across rotate; only key material changes
2. Without `--until` — immediate cutover (A-30 behavior preserved)
3. With `--until` — old verifying key remains for that `key_ref` until RFC3339 UTC
4. Fail-closed on invalid timestamps; no Manifesto/Meditation edits
5. Peer auto-notify stays Analyze-38

### Decision Drivers
1. QUEUE #4; blocked A-30 follow-up
2. Peer CRL grace already exists for *different* ids — node needs same-ref
3. Minimal persistence in `local.identity.json`

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Keyring `Vec` verifying + `previous_*` in identity JSON** | Clear; load/sync natural | JSON schema growth |
| B. Put old node key on CRL with grace (same id) | Reuse TrustStore | Conflicts with upsert of current; same-id CRL+entry messy |
| C. Sidecar only, Keyring unchanged | Small | Still can't verify same key_ref |

**Chosen: A.**

### ADR
- **Decision:** `Keyring.verifying: HashMap<String, Vec<VerifyingKey>>`; verify tries all. Rotate writes optional `previous_public_key` + `previous_grace_until`; `load_node_identity` / sync re-registers grace while active. CLI `identity rotate [--backup] [--until RFC3339]`.
- **Reject:** CRL for same node id; auto peer notify
- **Follow-up:** Analyze-38 peer pubkey notify

### Acceptance
- [x] Multi-key verify for same `key_ref`
- [x] Rotate without `--until`: old sig fails
- [x] Rotate with `--until` future: old + new verify; after expiry old fails on reload
- [x] CLI `--until`; docs + QUEUE
- [x] Tests + clippy
