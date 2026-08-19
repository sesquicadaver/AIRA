# Analyze-38 — Peer pubkey notify

**Scope:** After node secret rotate, notify address-book peers of the new pubkey (same `identity_id`). No gossip fanout.

**Status:** DONE — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Rekey announces **own** identity only (`subject_id == issuer`)
2. Reuse Noise + `peer.trust.delta` apply path (`--apply-trust`)
3. Opt-in notify (`--notify-peers`); **notify before cutover**; rotate must not fail if peers unreachable
4. Receiver upserts issuer pubkey (single TrustStore slot; no remote dual-key)
5. No Manifesto/Meditation; no DHT/gossip

### Decision Drivers
1. QUEUE #5 depends on #3 + #4
2. Same-id rekey ≠ peer `trust rotate` (different ids)
3. Minimal CLI surface

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. TrustDeltaOp::Rekey + notify-before-rotate + `--notify-peers`** | Reuses apply path; hello works | New op |
| B. Separate `peer.pubkey.notify` message type | Clear name | Duplicate parse/apply |
| C. Notify after rotate | Simpler CLI order | Hello fails (chicken/egg) |

**Chosen: A** (notify before cutover).

### ADR
- **Decision:** `rekey` op on `aira:peer:trust-delta:v1`; apply requires `subject==issuer` then `TrustStore::upsert`. `notify_peers_of_rekey(root, new_pubkey, …)` dials address book best-effort **before** `rotate_node_signing_secret`. CLI: `identity rotate --notify-peers`, `peer notify-rekey --pubkey-hex`.
- **Reject:** Gossip; notify-after-rotate; remote dual-key TrustStore
- **Follow-up:** Analyze-39 CSU publisher

### Acceptance
- [x] Rekey build/parse/apply (issuer-only)
- [x] Reject rekey when subject ≠ issuer
- [x] notify_peers best-effort roundtrip updates peer trust.json
- [x] CLI --notify-peers / notify-rekey
- [x] Tests + clippy; docs + QUEUE
