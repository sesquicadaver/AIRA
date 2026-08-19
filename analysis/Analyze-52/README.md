# Analyze-52 — Self-sovereign trust-delta apply

**Scope:** `apply_trust_delta` requires `subject_id == issuer` for all ops (Revoke/Unrevoke/Rotate/Rekey). No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR + UltraQA PASS)

## RALPLAN-DR

### Principles
1. Peer-applied trust mutations are self-sovereign only
2. Local CLI `trust revoke` remains the admin path for third parties
3. Gossip/relay still forward; apply enforces issuer==subject
4. Rekey behavior unchanged in spirit (already matched)
5. One micro — no schema/wire change

### Options
| Option | Verdict |
|--------|---------|
| **A. issuer==subject all ops** | Chosen (user) |
| B. Revoke exception for third parties | Rejected by user |
| C. Docs-only | Invalid |

### Pre-mortem
1. Breaks gossip demo of third-party CRL → rewrite tests to self-revoke/rekey
2. Operators expect mesh CRL → document local CLI path
3. Rotate subject≠new_id confusion → subject is old id and must equal issuer

### Acceptance
- [x] All ops IdentityMismatch when subject≠issuer
- [x] Self-ops succeed; tests updated (gossip/relay/revoke roundtrip)
- [x] docs/peer-link + QUEUE #17 + A-36 TODO
- [x] cargo test -p aira-peer + clippy

### Out
Third-party mesh CRL apply; STUN; mTLS optional; CN mapping.
