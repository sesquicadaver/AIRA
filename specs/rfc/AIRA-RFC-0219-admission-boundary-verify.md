# AIRA-RFC-0219 — Admission boundary verify (RFC-D)

## 1. Summary

QUEUE `#335` (Phase W / Pack 1 residual honesty): at every admission input boundary, `text` / `statement_content_hash` / `kind` / `payload_schema` must agree **before** ProblemSubmitted (or Context factor use). Forged or hand-mutated snapshots fail closed; HTTP maps to 4xx.

## 2. Problem Statement

`submit_problem_with_admission` accepted a caller-built `AdmissionSnapshot` without checking that `statement_content_hash` matched the submit `text`, or that `kind` / `payload_schema` were canonical. HTTP/CLI builders were honest; the library boundary was not (audit D6 / A4).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` A4; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `AdmissionSnapshot::verify_input_boundary(text)` on submit (before enforce-or-reject / effect)
- `FlowError::AdmissionBoundary` → HTTP 400
- Context CSU rejects admission factors with wrong `payload_schema`
- `verify_context_factor` helper for JSON factors

## 5. Non-Goals

```text
Activate evidence authority (#336)
Reuse candidate independent check (#340)
Result/verify task binding (#341)
Rewriting statement hash after admit
```

## 6. Compatibility

Snapshots from `from_text_and_constraints` / `default_for_text` pass unchanged. Hand-built mismatches reject.

## 7. Acceptance

```text
Forged statement_content_hash → AdmissionBoundary; no problem_ref.
Wrong kind/schema → reject.
Valid builder snapshot → admit OK.
Context factor with wrong schema → CSU error.
Tip → first OPEN #336.
```

## 8. References

- QUEUE `#335` · Analyze-372
- Parent: RFC-0215; prior RFC-0218
