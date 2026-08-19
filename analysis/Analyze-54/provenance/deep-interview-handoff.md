# Deep-interview handoff — Analyze-54

## Interview-complete rationale
User chose **D** after security tradeoff brief: hello already Ed25519-binds `x25519_pub_hex` each dial; dedicated x25519 peer-notify adds surface without admission gain. Ambiguity on purpose/apply-side cleared. Non-goals and decision boundaries set.

## Decisions
| Item | Choice |
|------|--------|
| QUEUE #19 | Close as **wont-need** (hello-sufficient) |
| Peer notify x25519 | **Not implemented** |
| Pin cache / dual-static grace | Out — future rows if ever needed |
| Next OPEN | #20 Analyze-55 (CN→TrustStore) |

## Non-goals
New message type; discovery write for x25519; handshake pin; dual-static grace; STUN; changing Ed25519 rekey notify.

## Acceptance
1. QUEUE #19 DONE with wont-need rationale
2. Analyze-54 CLOSED documenting D + Living Spec
3. A-49 TODO closed → wont-need
4. docs/peer-link + crypto Out updated (no #19 notify)
5. No runtime code change required (docs/QUEUE only)
