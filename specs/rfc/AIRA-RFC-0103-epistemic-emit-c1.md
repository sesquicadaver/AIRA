# AIRA-RFC-0103 — Epistemic emit on C1 2+2

## 1. Summary

Phase J `#207`: `OperationalPlane::submit_problem` Completed path must publish an epistemic-assessment artifact (`aira:schema:epistemic:assessment:0.1`) via `epistemic-basic` on `ResultPublished`. C1 `c1.pipeline.calculate_2_plus_2` asserts the body. Does **not** implement a full Epistemic plane.

## 5. Non-Goals

RFC-0096 (`#208`); full Epistemic plane; changing EPI-001 coordinates; promoting Epistemic Book rows to **DONE** beyond C1 emit.

## 10. Contract

```text
submit_problem Completed
  → latest_epistemic_assessment is Some
  → KnowledgeArtifact body validates as epistemic-assessment:0.1
missing assessment → FlowError (no Completed)
full Epistemic plane still out
```

## 15. Tests

```text
cargo test -p aira-flow --lib calculate_two_plus_two_emits_epistemic_assessment
cargo test -p aira-flow --lib calculate_two_plus_two_demo
cargo test -p aira-conformance --lib
```
