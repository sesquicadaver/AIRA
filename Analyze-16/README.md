# Analyze-16 — Epic 9 Conformance C0/C1

**Scope:** Issue Set #63–#70

## Ralplan (approved)

### Principles
1. Conformance Report matches `aira:schema:conformance:report:0.1` and is published as immutable CAS artifact
2. C0 runner: ontology + object/artifact immutability + event causality + policy gate
3. C1 runner: operational pipeline + CSU manifests + Verified Result completeness + failure-to-evidence
4. Individual tests #66–#70 are the concrete cases invoked by runners
5. No Manifesto/Meditation edits

### Acceptance
- #63 Report type + schema validation + immutable publish
- #64 C0 runner executes required suites + emits report
- #65 C1 runner executes required suites + emits report
- #66 Object mutation fails + InvariantViolation event
- #67 Artifact mutation fails + violation event
- #68 Event chain + causal_refs
- #69 Policy gate DENY without allowlist; enum limited
- #70 Failure → evidence, no fake Verified Result
- cargo test/fmt/clippy PASS

### Out of scope
Epic 10 protocols (#71+), demo docs (#76)
