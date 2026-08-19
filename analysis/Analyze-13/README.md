# Analyze-13 — Epic 6 Basic CSU Set

**Scope:** Issue Set #41–#46

## Ralplan (approved)

### Principles
1. Each basic CSU is an in-process `Csu` impl in `csu/*-basic`
2. Deterministic MVP only — no shell, no network, no ML
3. Artifacts via CAS; events via EventSink; isolation preserved
4. Output Artifact ≠ Verified Result Artifact
5. Failures become evidence (Evidence CSU)

### Decision
Implement all six basic CSUs with unit tests calling `on_event` + bound `CasArtifactStore`. Epic 7 end-to-end wiring stays out of scope.

### Acceptance
- #41 Context: ProblemSubmitted → Context Artifact + ContextResolved; marks ambiguity; no execute/result
- #42 Reduction: reuse check → Negative Lookup or Execution Capsule + ReductionCompleted
- #43 Execution: math.eval.safe / text.echo / text.uppercase; CapsuleCompleted|Failed; no shell/network
- #44 Verification: verify math output; Verified Result or VerificationFailed
- #45 Evidence: ResultPublished/CapsuleFailed/VerificationFailed → Evidence (+ FailureEvidenceCreated)
- #46 Artifact: publish/resolve/supersede via Artifact Runtime + corresponding events
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
Epic 7 operational flow (#47+), CLI problem submit, crypto verify.
