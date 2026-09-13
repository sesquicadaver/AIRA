# AIRA-RFC-0217 — Strict submit decoding (RFC-D)

## 1. Summary

QUEUE `#333` (Phase W / Pack 1 residual honesty): HTTP/Serde submit decoding must reject unknown fields with a client 4xx **before** execution. Text-only `{"text":…}` and known `admission` keys remain valid. Typos such as `admisson` or `model_reff` must not silently become default constraints.

## 2. Problem Statement

`ProblemSubmitBody`, `AdmissionConstraints`, `GenerationParameters`, `ResourceBudget`, and `FallbackRules` accepted unknown keys (Serde default). A misspelled top-level or nested field dropped the intended constraint and admitted a text-only (or partial) snapshot — audit D3.

## 3. Motivation

Post-V audit `d1115f2` / Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md) A2; parent consolidating RFC-0215 reserved until `#342`.

## 4. Scope

- `#[serde(deny_unknown_fields)]` on request constraint structs and `ProblemSubmitBody`
- Unit tests for unknown nested keys
- HTTP tests: typo / unknown → 4xx; text-only and known admission → OK
- Persisted `AdmissionSnapshot` remains permissive (forward-compatible store)

## 5. Non-Goals

```text
Constraints enforce-or-reject matrix (#334)
Admission boundary verify (#335)
Pack 2 GUI model select
Changing Axum Json status code family beyond client 4xx
```

## 6. Compatibility

Unconstrained C1 `Calculate 2 + 2` via text-only POST unchanged. Clients that send only documented keys are unaffected. Clients that relied on silent ignore of unknown keys must stop sending them.

## 7. Security / Integrity

Unknown keys fail closed at the request boundary so choice cannot be lost by typo into defaults before admit/execution.

## 8. Acceptance

```text
POST with unknown top-level or admission field → client 4xx; no completed/executed body.
POST {"text":"Calculate 2 + 2"} → completed (text-only OK).
POST with known admission.model_ref → still accepted (decode path).
Tip advances to first OPEN #334.
```

## 9. References

- QUEUE `#333` · Analyze-370
- Parent consolidating: RFC-0215 (file-free until `#342`)
- Prior: RFC-0210 (submit API constraints), RFC-0216 (math fidelity)
