# Analyze-39 — CSU emit_failed publisher

**Scope:** `CsuRuntime::emit_failed` and registry lifecycle events sign as `manifest.publisher_identity` (A-29 deferred). No peer work.

**Status:** CLOSED — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Runtime failure / lifecycle emits follow the same publisher contract as CSU business emits
2. Fail closed if publisher has no signing key (`signature_for`)
3. Default publisher == primary remains unchanged for stock manifests
4. OperationalPlane ProblemStatement stays on primary (out of scope)
5. No Manifesto/Meditation edits

### Decision Drivers
1. QUEUE #6 / A-29 TODO
2. Reuse `make_event_as`
3. Keep registry fallback to `with_event_identity` only if entry missing

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. emit_failed + lifecycle via publisher from manifest** | Consistent; closes deferred | Registry lifecycle changes |
| B. Only emit_failed | Smaller | Lifecycle still primary |
| C. Swap primary_signer around dispatch | Simple | Racy; rejected in A-29 |

**Chosen: A.**

### ADR
- **Decision:** `emit_failed` builds `CSUFailed` with `make_event_as(publisher, …, payload_ref=message)`. `emit_lifecycle` uses registered CSU’s `publisher_identity` via `make_event_as`; if no entry, keep previous primary identity pair.
- **Reject:** primary-only failure events; silent local-test fallback
- **Follow-up:** multi-tenant keyring (QUEUE #9)

### Acceptance
- [x] CSUFailed producer + signature.key_ref == publisher
- [x] Missing publisher signing key → fail closed
- [x] Lifecycle events for registered CSU use publisher
- [x] Existing default path still works
- [x] Tests + clippy; docs + QUEUE
