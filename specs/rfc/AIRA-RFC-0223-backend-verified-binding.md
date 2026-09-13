# AIRA-RFC-0223 — Backend verified binding (RFC-D)

## 1. Summary

QUEUE `#339` (Phase W / Pack 1 residual honesty): generate backends receive the activate-gate [`ExecutorFacts`] binding. **Mock must not stamp used-model.** Process with an explicit configured `expected_model_ref` that disagrees with the binding fail-closes. Used-model requires verified process binding acceptance.

## 2. Problem Statement

After `#328`, `stamp_executor_binding` copied activate-gate `model_ref` / `content_hash` onto every successful generate — including [`MockBackend`], which never runs weights. [`ProcessBackend`] ignored the binding and could stamp activated A while fixed argv selected B (audit D4 / A8).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` D4/A8; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `GenerateBackend::generate(payload, binding)`
- Mock: no `model_ref` / `model_content_hash`; stamp refuse if claimed
- Process: `with_expected_model_ref` mismatch → `BINDING_MISMATCH`; success stamps binding facts
- `ExecutorFacts.cache_path` from activated pointer
- Desktop `extract_used_model`: `backend==mock` → no used-model

## 5. Non-Goals

```text
Reuse candidate independent check (#340)
Result/verify task binding (#341)
Pack 2 multi-model GUI / real dual-model provenance matrix
Unlocking admission model_version enforcement beyond this atom
```

## 6. Compatibility

Reference plane / CI stay mock: CapsuleCompleted with capsule/problem refs, **without** used-model. Honest process + matching expected ref (or unset expected) stamps used-model as before for process paths.

## 7. Acceptance

```text
Mock generate → no model_ref / model_content_hash; GUI used-model empty
Mock claiming used-model → CapsuleFailed
Process expected_model_ref B + activated A → BINDING_MISMATCH CapsuleFailed
Process echo + AlwaysActivated → stamps model_ref/hash
Tip → first OPEN #340
```

## 8. References

- QUEUE `#339` · Analyze-376
- Parent: RFC-0215; prior RFC-0222; RFC-0213 executor facts; RFC-0172 backend≠model
