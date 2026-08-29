# AIRA-RFC-0081 — Conformance profile C4 scaffold (settlement receipts)

## 1. Summary

Phase H `#175`: `run_c4` in `aira-conformance` adds a local settlement audit scaffold with three named cases — receipt emit/verify-on-read, B2-011 privacy reject, and receipt link to a prior CRP route candidate. Not a merge gate; no blockchain ledger.

## 5. Non-Goals

CI job for C4; Settlement status PARTIAL (`#176`); C5 promotion (`#180`); remote settlement.

## 10. Named cases

```text
c4.settlement.receipt_emit_verify
c4.settlement.privacy_reject
c4.settlement.link_prior_route
```

## 15. Tests

```text
cargo test -p aira-conformance c4_
cargo run -p aira-cli -- conformance run --profile C4
```
