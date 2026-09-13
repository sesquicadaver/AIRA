# AIRA-RFC-0224 — Reuse candidate independent check (RFC-D)

## 1. Summary

QUEUE `#340` (Phase W / Pack 1 residual honesty): an admission-scoped reuse-index hit is **not** sufficient to Complete. [`admit_reuse_candidate`] independently checks artifact type, VERIFIED status (when present), and statement/result compatibility before Reduction binds a Ready Solution. Foreign VRA under a correct key must re-execute, not short-circuit.

## 2. Problem Statement

After `#326` / RFC-0211, `bind_catalog_for_admission` looked up `reuse_catalog_key` and called `enable_ready_solution` whenever `resolve` succeeded. A tampered or foreign verified payload under the right key published `reuse:ready_solution` → Completed without checking that the candidate matches this admit’s statement (audit A9 / D6).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` A9/D6; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `aira-flow::reuse::admit_reuse_candidate(admission, statement_text, type, payload)`
- Gate: VRA or ReadySolution; optional `verification_status == VERIFIED`; prefer `statement_content_hash` vs admission + text; else independent result vs math/echo/uppercase
- `OperationalPlane::bind_catalog_for_admission` resolves bytes and gates before `enable_ready_solution`; fail-closed → miss (re-execute)
- Compatible reuse under correct key remains Completed via `reuse:ready_solution`

## 5. Non-Goals

```text
Result/verify task binding (#341)
Consolidating RFC-0215 (#342)
Pack 2 multi-model GUI
Changing reuse_catalog_key composition (#326)
```

## 6. Compatibility

Default C1 `Calculate 2 + 2` reuse with a compatible Ready Solution still skips execution. Legacy text-only keys remain fail-closed (re-execute) as after `#326`.

## 7. Acceptance

```text
Correct key + foreign result (e.g. 999 for 2+2) → no reuse:ready_solution; re-execute → result 4
Compatible Ready Solution under correct key → Completed via reuse
statement_content_hash mismatch → miss
Tip → first OPEN #341
```

## 8. References

- QUEUE `#340` · Analyze-377
- Parent: RFC-0215; prior RFC-0223; RFC-0211 reuse-after-constraints; RFC-0100 reduction catalog bind
