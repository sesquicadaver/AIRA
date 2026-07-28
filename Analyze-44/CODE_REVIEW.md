# CODE_REVIEW — Analyze-44

## Scope
- `crates/aira-peer/src/{relay,address_book,session,lib}.rs`
- `crates/aira-cli` `--relay` / `relay-hold` / `add --via` / send via courier
- `docs/peer-link.md`, `QUEUE.md`, Analyze-44/**

## Code-reviewer lane
| Severity | Finding | Disposition |
|----------|---------|-------------|
| — | No CRITICAL/HIGH | — |
| LOW | Hub routes are in-memory only | By design (micro); durable store deferred |
| LOW | `--relay` ignores non-deliver envelopes on hub socket | Intentional courier-only |
| — | Anti-stub; original signature preserved; no Manifesto/Meditation; no STUN/DHT | OK |

**Recommendation: APPROVE**

## Architect lane
- Live registration is the correct NAT path vs dial-through-only
- Courier model matches A-43 provenance rule
- `via` keeps address book as dial source with explicit relay indirection

**Architectural status: CLEAR**

## Final verdict
**APPROVE** + **CLEAR**

## Evidence
`Analyze-44/verification/VERIFICATION.md`  
`Analyze-44/LIVING_SPEC_MATRIX.md`
