# ULTRAQA — Analyze-78

**Verdict:** PASS (local workspace)  
**Date:** 2026-08-19

## Goal
Canonical CSU manifests; mutation fails verify; C0/C1 green.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | crypto | mutate manifest fields | verify err | `canonical_verify_fails_when_manifest_fields_change` | PASS |
| U2 | registry | TESTSIG / unsigned | reject | security + unsigned tests | PASS |
| U3 | C1 | basic CSU manifests | validate_for_registration | `c1_suite_passes` | PASS |
| U4 | HTTP | tenant register after apply_publisher | 200 / 403 | tenant_register + http_csu_register | PASS |
| U5 | gate | workspace test + clippy `-D warnings` | green | exit 0 | PASS |
