# Analyze-11 — Epic 4 Artifact / Event / Policy

**Scope:** Issue Set #27–#34

## Ralplan (approved)

### Principles
1. Artifacts content-addressed + immutable; supersession = new artifact
2. Events append-only; causal_refs; no global total order
3. Policy returns only ALLOW|DENY|REQUIRE; unknown → DENY
4. InvariantChecker blocks + emits InvariantViolation event

### Decision
Full Epic 4 in `aira-artifact`, `aira-event`, `aira-policy` + `InvariantChecker` in `aira-core`.

### Acceptance
- ArtifactDescriptor ↔ schema; CAS by SHA-256; hash mismatch reject; mutation fails; supersession keeps old
- EventDescriptor ↔ schema; append-only; query by object/artifact ref; subscriptions; duplicate event_id idempotent
- Policy: ALLOW|DENY|REQUIRE; PolicyEvaluated event; unknown action DENY
- InvariantChecker: object/artifact immutability, event signature, policy-before-action; emit InvariantViolation event
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
CSU runtime (#35+), network protocols, crypto signature verify, claiming AIRA-C0 conformance harness pass.
