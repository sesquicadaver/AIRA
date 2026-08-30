# AIRA-RFC-0087 — Durable reuse index for LocalSession

## 1. Summary

Phase I `#189`: `LocalSession::submit_problem` seeds Reduction from `problems/reuse-index.json` (problem-text SHA-256 → artifact id). A repeat of the same text reuses the stored verified result and does not run Execution. Missing or unresolvable ids fall through to compute.

## 5. Non-Goals

Fail-closed signing (`#190`); atomic persist (`#191`); knowledge-catalog reuse; changing `enable_ready_solution` pre-seed tests.

## 10. Contract

```text
key ← sha256(problem text)
first Completed → reuse-index.by_content_hash[key] = verified_artifact_id (first wins)
later submit with same text → open_with_ready_nonce([id]) if CAS resolve succeeds
reuse → ReductionCompleted/ResultPublished payload reuse:ready_solution; no CapsuleCompleted
different text → no reuse
```

## 15. Tests

```text
cargo test -p aira-flow --lib local_session_repeat_problem_reuses_without_execution
cargo test -p aira-flow --lib
```
