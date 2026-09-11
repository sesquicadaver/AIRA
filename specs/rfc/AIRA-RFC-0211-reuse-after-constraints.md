# AIRA-RFC-0211 — Reuse after admission constraints (RFC-D)

## 1. Summary

QUEUE `#326` (Phase V / Repair Pack 1): the durable reuse-index key is derived from the immutable [`AdmissionSnapshot`] identity (`reuse_catalog_key`), not from problem text alone. [`ReusePolicy::RequireNewExecution`] skips both lookup and record. Legacy text-only keys fail closed (re-execute).

## 2. Problem Statement

`#325` frozen constraints at admit, but `reuse-index.json` still keyed by text SHA (`#204`). A Completed result under default admit could satisfy a later admit that required a different `model_ref` / generation / placement.

## 3. Motivation

`aira-repair.md` Pack 1; [`docs/phase-v-plan.md`](../../docs/phase-v-plan.md); parent consolidating RFC-0208 reserved until `#330`.

## 4. Scope

- `AdmissionSnapshot::reuse_catalog_key` — composite identity hash; `None` when require-new
- `reuse.rs` lookup/record by admission key; ignore legacy text-only entries
- `OperationalPlane::bind_catalog_for_admission` before publish
- `LocalSession` records Completed under the admit key
- Tests: same text + different `model_ref` → no reuse; `RequireNewExecution` → re-execute; default C1 text reuse still works

## 5. Non-Goals

```text
activate_verified hash continuity (#327) — DONE @ RFC-0212
Capsule↔Output↔Result binding (#328)
Pack 2 multi-model GUI
GPU marketplace / LLM-in-Core
```

## 6. Compatibility

On-disk field name `by_content_hash` retained. Values after `#326` are admission keys. Pre-`#326` text-only keys are not looked up (fail-closed).

## 7. Security / Integrity

Reuse may apply only when snapshot constraints match. Compare/measure admits use `RequireNewExecution` so they never hit Success from cache.

## 8. Acceptance

```text
reuse_catalog_key ⊇ statement + model/generation/placement/privacy/budget/fallback.
Same text + different model_ref → CapsuleCompleted (no reuse:ready_solution).
RequireNewExecution → no catalog hit.
Default text-only C1 Calculate 2 + 2 still reuses across submits.
```

## 9. References

- QUEUE `#326` · Analyze-363
- Depends on RFC-0209 (`#324`), RFC-0210 (`#325`)
- Parent consolidating: RFC-0208 (file-free until `#330`)
