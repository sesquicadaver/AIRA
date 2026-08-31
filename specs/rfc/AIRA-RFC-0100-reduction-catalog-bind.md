# AIRA-RFC-0100 — Reduction catalog bind

## 1. Summary

Phase J `#204`: `OperationalPlane` binds `problems/reuse-index.json` (RFC-0087). On `submit_problem`, Reduction reuses a resolvable artifact id for that problem text. Tests and `LocalSession` do **not** call `enable_ready_solution` for this path.

## 5. Non-Goals

Semantic verify `text.*` (`#205`); knowledge-vec catalog beyond reuse-index; RFC-0096 (`#208`); changing `drain_from`.

## 10. Contract

```text
open_with_reuse_index(artifacts, reuse-index.json)
submit_problem(text) → lookup sha256(text) → CAS resolve → Reduction reuse:ready_solution
no CapsuleCreated / CapsuleCompleted on hit
miss / unresolvable id → compute as today
enable_ready_solution — in-memory helper only, not the durable path
```

## 15. Tests

```text
cargo test -p aira-flow --lib plane_reduction_binds_reuse_index_without_enable_ready_solution
cargo test -p aira-flow --lib ready_solution_reuse_skips_execution
cargo test -p aira-flow --lib local_session_repeat_problem_reuses_without_execution
```
