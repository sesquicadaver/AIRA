# AIRA-RFC-0218 — Constraints enforce-or-reject (RFC-D)

## 1. Summary

QUEUE `#334` (Phase W / Pack 1 residual honesty): every non-default `AdmissionConstraints` field is either **enforced** at runtime or rejected as **explicit unsupported** before effect. Unconstrained C1 `Calculate 2 + 2` stays deterministic. `placement=remote_required`, generation knobs, budgets, and model constraints on math fail closed with a clear error (HTTP 4xx).

## 2. Problem Statement

Admission fields participated mainly in the reuse key and Serde copy. `remote_required` still ran locally; `model_ref` on math was ignored; generation/privacy/budget/fallback flags were stored without effect (audit D2 / A3).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` D2; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `AdmissionSnapshot::enforce_or_reject` matrix called at submit admit
- `FlowError::UnsupportedConstraint` → HTTP `400`
- `problem_binds_math_eval_safe` export for path-aware gating
- Tests: remote_required / math+model_ref / generation → reject; text-only OK

## 5. Matrix (non-default)

| Field | Outcome |
|-------|---------|
| `reuse_policy` | **enforced** (catalog key / skip) |
| `placement=local\|remote_allowed` | **enforced** (local permitted) |
| `placement=remote_required` | **unsupported** |
| `model_ref` / `allowed_model_refs` on math | **unsupported** |
| `model_ref` on generate | **enforced** at activate gate |
| `allowed_model_refs` without `model_ref` | **unsupported** (auto-within-set later) |
| `model_version` / `model_content_hash` | **unsupported** |
| `generation.*` | **unsupported** |
| `privacy_class` / `resource_budget.*` | **unsupported** |
| `fallback.allow_*=true` | **unsupported** |
| `fallback` both false | **enforced** (default) |

## 6. Non-Goals

```text
Admission boundary text/hash verify (#335)
Activate evidence authority (#336)
Backend verified binding / generation knobs apply (#339)
Pack 2 GUI / remote Pack 4
```

## 7. Compatibility

Text-only admits unchanged. Clients that sent model/generation on math must omit them or use generate-local with an activated matching model.

## 8. Acceptance

```text
remote_required → UnsupportedConstraint / HTTP 400 before execution.
Calculate 2+2 + model_ref → reject (not silent Completed).
Text-only Calculate 2+2 → VERIFIED 4.
Tip → first OPEN #335.
```

## 9. References

- QUEUE `#334` · Analyze-371
- Parent: RFC-0215 (file-free until `#342`); prior RFC-0217
