# AIRA-RFC-0072 — C2 event publish idempotency case

## 1. Summary

Phase G `#122`: conformance case `c2.event.publish_idempotent` in `run_c2` — duplicate `EventProtocolAdapter::publish_event` returns ACCEPTED without second log append.

## 5. Non-Goals

Wire network EP; changing idempotency semantics.

## 15. Tests

`cargo run -p aira-cli -- conformance run --profile C2`
