# AIRA-RFC-0083 — Conformance profile C5 scaffold (research separation)

## 1. Summary

Phase H `#180`: `run_c5` in `aira-conformance` adds a local research-separation scaffold with three named cases — research event/artifact rejected as operational input, promotion-candidate event rejected, and promotion-candidate schema fixtures. Not a merge gate; CAS publish of research artifacts remains allowed.

## 5. Non-Goals

CI job for C5; promotion status rollup (`#181`); promoting any research item; canary deploy; Core/ABI change.

## 10. Named cases

```text
c5.research.separation
c5.promotion.gate_reject
c5.promotion.candidate_schema
```

## 15. Tests

```text
cargo test -p aira-conformance c5_
cargo run -p aira-cli -- conformance run --profile C5
```
