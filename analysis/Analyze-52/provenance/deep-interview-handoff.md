# Deep-interview handoff — Analyze-52

## Interview-complete rationale
User answered **A** (self-sovereign only). Ambiguity cleared: third-party CRL via gossip/apply is intentionally closed; mesh fanout remains for self-signed deltas only.

## Decisions
| Decision | Choice |
|----------|--------|
| apply_trust_delta | subject_id == issuer for Revoke, Unrevoke, Rotate, Rekey |
| Third-party revoke | Reject IdentityMismatch |
| Local CLI trust revoke | Unchanged (not peer-delta) |
| Gossip | Still forwards original signed delta; apply fails if subject≠issuer |

## Non-goals
Optional mTLS; CN mapping; STUN/discv5; Manifesto/Meditation; changing local `trust revoke` CLI.

## Acceptance
1. subject≠issuer → IdentityMismatch for all ops
2. subject==issuer self-ops succeed (revoke/rotate/rekey/unrevoke)
3. Gossip/relay tests use self-announce
4. Docs + QUEUE + A-36 TODO; tests+clippy
