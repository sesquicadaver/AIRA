# AIRA-RFC-0237 — Work Compare mode (RFC-D)

## 1. Summary

QUEUE `#354` (Phase X / Pack 2 X2): Work executor gains **Compare / Порівняти** — two distinct available `model_ref`s, sequential dual submit with `ReusePolicy::RequireNewExecution`, fail-closed when either leg is empty/identical/unready, **no silent substitute**. Not Pack 3 ratings.

## 2. Problem Statement

Pack 2 GUI required Auto / Specific / Compare, but Desktop only admitted one model per submit. Users could not compare two real models on the same draft without inventing a substitute path.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2 after Help F1 (#353). PR-M4: Compare without silent substitute. Parent RFC-0226 reserved until `#358`.

## 4. Scope

- `WorkExecutorPreference::Compare { a, b }` + dual readiness / admissions
- Work UI: Compare radio, A/B pickers, stacked A|B results (B failure keeps A)
- Sequential dual job in one async submit slot (`try_spawn_compare_submit`)
- Help F1: Compare is an available Work order (not a non-order)
- Tip → first OPEN `#355`

## 5. Non-Goals

```text
Model data paths UI (#355)
Apply-diff (#356)
Two-model installed e2e M6 (#357)
Pack 3 ratings / measured quality
Parallel multi-submit / Core dual-admit schema
Silent tip fallback for unready Compare legs
```

## 6. Compatibility

Single Auto/Specific path unchanged. Compare admissions set `RequireNewExecution` so cache Success from another model cannot satisfy a leg.

## 7. Acceptance

```text
Compare blocked when A/B empty, A==B, either unready, or draft is math
Both ready → two RequireNewExecution admissions with distinct model_ref
B failure surfaces explicitly; A result retained
RFC-0237 + tip first OPEN #355
```

## 8. References

- QUEUE `#354` · Analyze-391
- Parent: RFC-0226; RFC-0232 Work readiness; RFC-0233 mock triple; Pack 2 PR-M4
