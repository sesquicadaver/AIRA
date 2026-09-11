# AIRA-RFC-0216 — Math capsule fidelity (RFC-D)

## 1. Summary

QUEUE `#332` (Phase W / Pack 1 residual honesty): math Problem→Capsule must carry the stated expression. Never invent a default `2+2`. Bare `9-3` / `9/3` / `42` and `Calculate …` forms evaluate and VERIFIED to the correct result; unsupported math statements fail closed without a substituted VRA.

## 2. Problem Statement

`is_math_eval_safe` accepted subtraction/division/bare numbers, but reduction defaulted the capsule expression to `2+2` when `+`/`*` were absent. Verification then independently confirmed the substituted capsule → VERIFIED 4 for a different problem (audit D1).

## 3. Motivation

Post-V audit `d1115f2` / reaudit `70b525d`; [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); parent consolidating RFC-0215 reserved until `#342`.

## 4. Scope

- `extract_math_expression` in `reduction-basic` (no default `2+2`)
- CapsuleFailed when classified math cannot extract a pure expression
- `execution-basic` refuses missing expression (no `unwrap_or("2+2")`)
- Tests: unit extract; plane `9-3`→6, `9/3`→3, `42`→42; unsupported ≠ VERIFIED

## 5. Non-Goals

```text
Strict HTTP unknown fields (#333)
Constraints enforce-or-reject (#334)
Pack 2 GUI model select
Parser depth budget (S9) as a separate atom
```

## 6. Compatibility

Unconstrained C1 `Calculate 2 + 2` remains `math.eval.safe` → VERIFIED 4. Echo/uppercase/generate-local binds unchanged.

## 7. Security / Integrity

Verification still checks the capsule expression it is given; fidelity requires that expression match the admitted problem text extract — not a silent substitute.

## 8. Acceptance

```text
9-3 → VERIFIED 6; 9/3 → VERIFIED 3; 42 → VERIFIED 42.
Unsupported Calculate… → CapsuleFailed; no VERIFIED 4 from default 2+2.
Calculate 2 + 2 unchanged.
Tip advances to first OPEN #333.
```

## 9. References

- QUEUE `#332` · Analyze-369
- Parent consolidating: RFC-0215 (file-free until `#342`)
