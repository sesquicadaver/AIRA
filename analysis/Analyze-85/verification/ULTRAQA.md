# ULTRAQA — Analyze-85

**Verdict:** PASS (local)  
**Date:** 2026-08-19

## Goal
TLS/mTLS behavior unchanged after file split.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | self-signed | load rustls config | ALPN http/1.1 | `self_signed_loads_into_rustls_config` | PASS |
| U2 | mTLS | trusted CN handshake | ok | `mtls_accepts_trusted_cn` | PASS |
| U3 | mTLS | unknown / revoked / no cert / wrong CA | reject | `mtls_rejects_*` | PASS |
| U4 | CA | empty PEM | fail-closed | `client_ca_empty_fails_closed` | PASS |
| U5 | clippy | aira-node -D warnings | green | cargo clippy -p aira-node | PASS |
