# AIRA-RFC-0227 — Per-model inventory lifecycle (RFC-D)

## 1. Summary

QUEUE `#344` (Phase X / Pack 2 M1): verified and available states are **per-model** and survive when `verified.latest` / `activated.latest` tip moves. Latest pointers are the default selection tip only — **not** sole generate-local execution authority when `model_artifact_ref` names another available model.

## 2. Problem Statement

Verify/activate wrote only `models/*.latest.json`. A second verify/activate overwrote the tip, so model A’s lifecycle was not independently durable. `ActivatedPointerGate` admitted solely against `activated.latest`, so a non-latest available model could not execute even with a correct `model_artifact_ref`.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) M1; Pack 2 / audit inventory lifecycle; parent RFC-0226 reserved until `#358`.

## 4. Scope

- Per-model slot locators: `models/verified/<slot>/pointer.json`, `models/cache/<slot>/activated.json`
- `verify_quarantine` / `activate_verified` / `activate_verified_model` write slots without clearing other models
- `list_model_lifecycle` / `load_verified_slot` / `load_activated_slot` for restart-stable A/B state
- `ActivatedPointerGate::check_activated`: requested `model_artifact_ref` resolves slot first; absent ref → latest tip

## 5. Non-Goals

```text
Model select API Auto/required (#345)
Settings Models catalog GUI (#348)
Work executor / Compare
Weakening hash/evidence admit
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

Roots without slot files still admit when `model_artifact_ref` matches `activated.latest`. Default (no ref) still uses latest tip. Evidence/hash fail-closed unchanged.

## 7. Acceptance

```text
Verify A then B → both verified slots; latest tip = B; A still verified
Activate A then B → both available slots; latest tip = B; A still available
list_model_lifecycle after reopen → A and B verified+available
Gate: latest=B, request A via model_artifact_ref → admit A
Tip → first OPEN #345
```

## 8. References

- QUEUE `#344` · Analyze-381
- Parent: RFC-0226; prior Phase X wiring `#343`; RFC-0109 activate gate; RFC-0013 activate-verified
