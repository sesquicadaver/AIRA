# AIRA-RFC-0101 — Semantic verification for text.echo / text.uppercase

## 1. Summary

Phase J `#205`: VerificationBasic independently compares claimed string `result` to `expression` from the output body or the capsule artifact (`CapsuleCompleted` second `artifact_ref`). A wrong string is not VERIFIED (`VerificationFailed`). Does not import execution-basic (CSU ↛ CSU).

## 5. Non-Goals

Evidence primacy runtime (`#206`); epistemic emit (`#207`); RFC-0096 (`#208`); changing execution-basic or reduction-basic.

## 10. Contract

```text
expression ← output.expression OR capsule.expression
text.echo      VERIFIED iff claimed result string == expression
text.uppercase VERIFIED iff claimed result string == expression.to_uppercase()
wrong string (e.g. echo hello claimed "world") → VerificationFailed
missing expression → VerificationFailed
```

## 15. Tests

```text
cargo test -p aira-csu-verification-basic --lib wrong_text_echo_result_is_not_verified
cargo test -p aira-csu-verification-basic --lib wrong_text_uppercase_result_is_not_verified
cargo test -p aira-csu-verification-basic --lib
```
