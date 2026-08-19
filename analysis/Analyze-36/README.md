# Analyze-36 — Trust-delta over peer

**Scope:** CRL / trust-delta messages over Noise-encrypted peer link; explicit send + optional apply. No auto-notify UX, no gossip/DHT.

**Status:** DONE — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Fail-closed: only verified envelopes from authenticated peers; refuse ops on `local-test` and local node id
2. Encrypt via existing Noise transport (no cleartext delta)
3. Explicit apply (`--apply-trust`); no silent mutation of `trust.json` without operator opt-in
4. Reuse `TrustStore::{revoke,rotate,unrevoke}` — no parallel CRL logic
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### Decision Drivers
1. QUEUE #3 after Noise XX
2. Minimal wire + CLI surface for demo
3. Keep Analyze-38 (auto-notify) separate

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. `peer.trust.delta` + trust-send + listen `--apply-trust`** | Explicit; testable; fail-closed | Operator must opt-in apply |
| B. Auto-apply on every recv | Less CLI | Silent trust mutation; UX scope creep |
| C. Full trust.json snapshot sync | Complete mirror | Large; merge conflicts; overkill |

**Chosen: A.**

### ADR
- **Decision:** Message type `peer.trust.delta`; payload JSON schema `aira:peer:trust-delta:v1` with ops `revoke` | `rotate` | `unrevoke`. Signed envelope like ping (`payload_hash` over JSON bytes). CLI `peer trust-send`; listen `--recv --apply-trust` applies after verify + sync_trust_verifiers.
- **Reject:** Auto-apply default; gossip fanout; auto-notify on local rotate (→ #5)
- **Follow-up:** Analyze-38 peer pubkey notify; dual-key grace (#4) remains orthogonal

### Acceptance
- [x] Build/parse/apply trust-delta (revoke/rotate/unrevoke)
- [x] Refuse local-test and local-node subject ops
- [x] Encrypted roundtrip + apply updates trust.json
- [x] CLI trust-send + listen --apply-trust
- [x] Tests + clippy; docs + QUEUE update
