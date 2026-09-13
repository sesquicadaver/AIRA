# AIRA-RFC-0225 — Result/verify task binding (RFC-D)

## 1. Summary

QUEUE `#341` (Phase W / Pack 1 residual honesty): result read and verification bind to the admitted problem independently of index locators and CapsuleCompleted artifact pairing. Swapped foreign VRA locators fail closed. Generate-local capsules without math `expression` must not emit a false `VerificationFailed`.

## 2. Problem Statement

After `#315`, `get_result(problem_id)` resolved the index locator via ArtifactStore hash verify but did not check that the body belongs to that problem — swapping `verified_artifact_id` to another valid VRA returned the foreign body (audit A10). Verification (`#314`) sourced action/expression from the admitted capsule but did not require `capsule.problem_statement_ref == event.object_refs[0]`, and required `expression` before the generate-local early-return — production generate capsules carry `prompt` only → false `VerificationFailed` (S11 / A10).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` A10/D6; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `LocalSession::get_result`: after resolve, [`artifact_binds_problem_lookup`] (problem ref / statement hash / independent result check)
- `VerificationBasicCsu`: capsule↔event problem binding; generate-local action short-circuit before requiring `expression`
- Compatible same-statement reuse (new problem id, shared VRA) remains readable via result compatibility

## 5. Non-Goals

```text
Consolidating RFC-0215 / QUEUE W close (#342)
Pack 2 multi-model GUI
Changing reuse_catalog_key / admit_reuse_candidate (#340)
Re-minting VRA problem_statement_ref on reuse bind
```

## 6. Compatibility

Honest `get_result` after Completed/Executed unchanged. Reuse of a same-text Ready Solution across problem ids still resolves. Capsule fixtures without `problem_statement_ref` still verify when expression matches.

## 7. Acceptance

```text
Swap verified_artifact_id to foreign VRA → get_result fail-closed
Capsule problem_statement_ref ≠ event object_refs → VerificationFailed
Generate-local capsule with prompt only → no VerificationFailed; not VERIFIED
Tip → first OPEN #342
```

## 8. References

- QUEUE `#341` · Analyze-378
- Parent: RFC-0215; prior RFC-0224; RFC-0199 capsule-sourced verify; RFC-0200 result-by-problem
