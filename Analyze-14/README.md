# Analyze-14 — Epic 7 Local Operational Flow

**Scope:** Issue Set #47–#56

## Ralplan (approved)

### Principles
1. Problem submit creates immutable ProblemStatement object + ProblemSubmitted
2. Event drain dispatches to Active CSUs with ArtifactStore bound
3. Reuse before compute; failures become evidence; no auto normative collapse
4. Demos prove Calculate 2+2, reuse, failure-to-evidence, DSF stub

### Decision
New `aira-flow` crate (`OperationalPlane`) wires Epic 6 CSUs. Extend `CsuRuntime` with artifact-bound dispatch. Library API for #47; full CLI problem commands remain Epic 8.

### Acceptance
- #47 submit → object + ProblemSubmitted (schema-valid)
- #48–#52 wired pipeline Context→Reduction→Execution→Verification→Evidence
- #53 Calculate 2+2 → result 4, VERIFIED, confidence 1.0, events queryable
- #54 Ready Solution reuse → no CapsuleCompleted from execution; ResultPublished
- #55 missing input → CapsuleFailed + Failure Evidence; no Verified Result
- #56 two alternatives → Differentiated Solution Field; requires_human_collapse=true
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
Epic 8 full CLI/node (#57–#62), conformance runners (#63+).
