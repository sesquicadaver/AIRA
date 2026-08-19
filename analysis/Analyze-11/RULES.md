# Правила Analyze-11

Immutability + MVP freeze + Event-native / Artifact-based contracts.

## Scope
Issue #27–#34 only (Epic 4).

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Artifacts are CAS + immutable; mutation API must fail
3. Events are append-only; no global total order required
4. Policy unknown controlled action → DENY
5. Invariant violations emit `InvariantViolation` events via `EventSink`

## Out of scope
CSU dispatch (#35+), network protocols, cryptographic signature verification, full conformance harness.
