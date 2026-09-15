# AIRA-RFC-0232 — Work executor + capability readiness (RFC-D)

## 1. Summary

QUEUE `#349` (Phase X / Pack 2 X2): Work screen exposes **Auto / specific** executor preference and **pre-submit readiness** that distinguishes deterministic math from local text generation.

## 2. Problem Statement

Settings → Models (`#348`) can list/select/prepare models, but Work still submitted without binding admission `model_ref` or explaining when generate-local cannot run. Math must stay unblocked without a model; generate must fail closed when tip/required is unavailable. Choice of model still ≠ VERIFIED.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2 path: catalog → Work executor → mock honesty. Parent RFC-0226 reserved until `#358`.

## 4. Scope

- Runtime facade `work_readiness`: `WorkExecutorPreference::{Auto, Required}`, `WorkCapabilityKind::{Math, Generate}`, `evaluate_work_readiness` → `AdmissionConstraints`
- Math (`problem_binds_math_eval_safe`) → always ready; omit `model_ref`
- Generate → Auto tip or Required available catalog/tip; unready blocks submit with `work.model_unready`
- Desktop Work: Auto/specific UI + readiness reasons; `submit_problem_with_admission`
- Choice ≠ VERIFIED (copy + reasons)

## 5. Non-Goals

```text
Mock honesty / result triple (#350)
Compare mode (#354)
Remote download / marketplace
Weakening activate evidence/hash admit
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

Settings catalog remains the source of tip/rows. CLI/HTTP admission constraints unchanged. Math + `model_ref` still rejected at admit (Work omits model for math).

## 7. Acceptance

```text
Calculate 2 + 2 → ready without model; admission.model_ref = None
generate Auto without tip → not ready; submit blocked
generate Required empty → not ready
generate Auto/Required available → ready; admission.model_ref set
UI shows Auto/specific + readiness; choice ≠ VERIFIED
Tip → first OPEN #350
```

## 8. References

- QUEUE `#349` · Analyze-386
- Parent: RFC-0226; prior RFC-0231 Settings catalog; RFC-0228 select; RFC-0210 admission
