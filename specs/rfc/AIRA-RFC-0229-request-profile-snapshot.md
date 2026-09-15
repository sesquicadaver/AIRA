# AIRA-RFC-0229 — Request profile → admission snapshot (RFC-D)

## 1. Summary

QUEUE `#346` (Phase X / Pack 2 M3): request-time **allowed** / **excluded** model sets freeze into an immutable [`AdmissionSnapshot`] at admit. Auto-within-set (`allowed_model_refs` minus `excluded_model_refs`) becomes a concrete `model_ref`. Live Settings-like mutation after submit must not rewrite the admitted snapshot.

## 2. Problem Statement

RFC-0218 left `allowed_model_refs` without `model_ref` as **unsupported** (“auto-within-set later”). Excludes had no admit field. A caller could copy constraints after submit and appear to change the task; Auto could not record a frozen choice from a delegated set.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) M3; PR-M2 exclude-from-auto; parent RFC-0226 reserved until `#358`.

## 4. Scope

- `AdmissionConstraints` / `AdmissionSnapshot.excluded_model_refs`
- `from_text_and_constraints` freezes Auto-within-set (sorted remaining allowed)
- Empty remainder / required∩excluded / excludes on math → fail closed
- `select_model_with_profile` honors the same allowed/excluded set
- Mid-flight Settings mutation ≠ `last_admission`

## 5. Non-Goals

```text
CLI supported-contract parity / --exclude flags (#347)
Settings Models catalog GUI (#348)
Work executor / Compare
Weakening activate evidence/hash admit
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

Text-only admits unchanged. RFC-0218 cell “allowed without model_ref = unsupported” is **superseded** for generate-local: Auto-within-set is now enforced by freeze. Math still rejects model/allowed/excluded. HTTP `admission.excluded_model_refs` is a known constraint key (`deny_unknown_fields`).

## 7. Acceptance

```text
allowed {A,B} exclude A → snapshot.model_ref = B; excludes persist
allowed {A} exclude A → reject (empty Auto-within-set)
required A + exclude A → reject
exclude on Calculate 2+2 → reject
mutate constraints after admit → last_admission unchanged
select Auto with exclude(tip) → remaining available, not tip
Tip → first OPEN #347
```

## 8. References

- QUEUE `#346` · Analyze-383
- Parent: RFC-0226; prior RFC-0228 select API; RFC-0218 enforce-or-reject; RFC-0210 submit constraints
