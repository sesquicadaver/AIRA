# AIRA-RFC-0074 — C2 envelope unsigned reject

## 1. Summary

Phase G `#124`: conformance case `c2.protocol.envelope_unsigned` — unsigned envelope fixture fails schema; empty/TESTSIG `signature_value` fails `ProtocolEnvelope::validate_signature`.

## 5. Non-Goals

Envelope semantics change; wire network.

## 15. Tests

`cargo run -p aira-cli -- conformance run --profile C2`
