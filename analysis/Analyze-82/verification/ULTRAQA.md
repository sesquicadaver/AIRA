# ULTRAQA — Analyze-82

**Verdict:** PASS (local)  
**Date:** 2026-08-19

## Goal
Crypto behavior unchanged after file split.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | sign | local-test roundtrip / TESTSIG | same | `local_test_sign_verify_roundtrip` | PASS |
| U2 | trust | CRL revoke / rotate / rekey | same | trust_* tests | PASS |
| U3 | rotate | node backup / grace / prune | same | node_rotate_* / prune_* | PASS |
| U4 | tenant | isolation / prune | same (no tenant.rs edit) | tenant::tests | PASS |
| U5 | clippy | aira-object -D warnings | green | cargo clippy -p aira-object | PASS |
