# AIRA-RFC-0085 — Semantic verification for math.eval.safe

## 1. Summary

Phase I `#187`: VerificationBasic independently evaluates the `math.eval.safe` expression from the output body or the capsule artifact (`CapsuleCompleted` second `artifact_ref`). A wrong finite claimed `result` is not VERIFIED (`VerificationFailed`). Does not import execution-basic (CSU ↛ CSU).

## 5. Non-Goals

PolicyGate in invoke (`#188`); durable reuse (`#189`); changing text.echo / text.uppercase beyond presence; Core/ABI Handle.

## 10. Contract

```text
expression ← output.expression OR capsule.expression
computed ← independent math_eval_safe(expression)
VERIFIED only if claimed result is finite AND |claimed - computed| ≤ 1e-9
wrong finite (e.g. 2+2 claimed 5.0) → VerificationFailed
```

## 15. Tests

```text
cargo test -p aira-csu-verification-basic --lib
cargo test -p aira-flow --lib
cargo test -p aira-conformance --lib
```
