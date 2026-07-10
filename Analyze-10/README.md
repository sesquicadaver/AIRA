# Analyze-10 — C0 Core Runtime (Epic 3)

**Scope:** Issue Set #22–#26

## Ralplan (approved)

### Principles
1. Core types only — no domain/ML/GPU/Node ontology
2. Opaque Handle — no storage path leakage
3. Object immutability — mutation → InvariantViolation
4. Schema-aligned ObjectDescriptor (aira-schema)

### Decision
Epic 3 only (not Epic 4 Artifact/Event/Policy). Memory + SQLite ObjectStore.

### Acceptance
- AiraRef/Hash/Signature serde tests
- Handle opacity test
- ObjectDescriptor schema + forbidden types
- create/open; mutate fails
- SQLite objects table; lookup; duplicate deterministic
- cargo test/fmt/clippy PASS; originals untouched
