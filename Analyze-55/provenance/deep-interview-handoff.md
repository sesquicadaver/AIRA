# Deep-interview handoff — Analyze-55

## Interview-complete rationale
User required simple/transparent/reliable/low-cost protocol. Recommended **A** (CN = full AiraRef → TrustStore). User continued Autopilot without rejecting A; constraints lock B/C out as over-complex. Ambiguity cleared.

## Decisions
| Item | Choice |
|------|--------|
| Identity source | Client cert **CN** == full `aira:identity:…` |
| Check | TrustStore entry exists + not revoked |
| When | Always when `--tls-client-ca` (mTLS) |
| Enforcement | Handshake verifier (after CA), fail-closed |
| SAN / short-name map | Out |

## Non-goals
Optional client auth; separate health listener (#21); Bearer changes; public bind; SAN priority; short CN rewrite.

## Acceptance
1. Valid CA + CN in TrustStore → handshake ok
2. Valid CA + unknown/revoked/invalid CN → reject
3. Docs local-node + QUEUE #20; tests + clippy
