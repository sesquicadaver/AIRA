# AIRA-RFC-0213 — Capsule↔Output↔Result binding / executor facts (RFC-D)

## 1. Summary

QUEUE `#328` (Phase V / Repair Pack 1): generate-local ExecutionArtifact must carry proven executor facts (`model_ref` + `model_content_hash`) from Phase D activate binding, plus capsule/problem refs. CapsuleCompleted continues to bind `[output, capsule]`. Admission `model_ref` is copied into the generate-local capsule when present. Chosen ≠ executed without these stamps.

## 2. Problem Statement

Structural CapsuleCompleted linking existed, but Mock/Process outputs stamped only `backend`. Activate gate returned `Ok(())` without identity. UI/result paths could not prove which weights ran.

## 3. Motivation

`aira-repair.md` Pack 1; [`docs/phase-v-plan.md`](../../docs/phase-v-plan.md); PR-M3 partial; parent consolidating RFC-0208 reserved until `#330`.

## 4. Scope

- `ExecutorFacts` + `ModelActivateGate::check_activated` → `Result<ExecutorFacts, String>`
- `ActivatedPointerGate` / `AlwaysActivated` return model_ref + content_hash
- `ExecutionLlmCsu` stamps facts + `capsule_ref` / `problem_statement_ref` on output; conflict → CapsuleFailed
- Reduction copies admission `model_ref` → capsule `model_artifact_ref`
- Tests: stamp present; conflict fail-closed; admission→capsule; plane Executed carries facts

## 5. Non-Goals

```text
E2E Pack 1 suite (#329)
Pack 2 multi-model GUI / «Порівняти»
Minting VERIFIED for generate-local
GPU marketplace / LLM-in-Core
```

## 6. Compatibility

Capsule schema unchanged (`model_artifact_ref` already optional). Output gains additional fields; clients that ignore unknown keys remain valid.

## 7. Security / Integrity

Executor facts come from the activate gate after weight hash admit — not from backend self-report. Conflicting claimed `model_ref` / `model_content_hash` fail closed.

## 8. Acceptance

```text
generate-local ExecutionArtifact has model_ref + model_content_hash + capsule_ref.
CapsuleCompleted.artifact_refs = [output, capsule].
Admission model_ref appears on capsule as model_artifact_ref when set.
Forged backend model_ref → CapsuleFailed.
C1 Calculate 2 + 2 unchanged.
```

## 9. References

- QUEUE `#328` · Analyze-365
- Depends on RFC-0105 / RFC-0109 / RFC-0210–0212
- Parent consolidating: RFC-0208 (file-free until `#330`)
