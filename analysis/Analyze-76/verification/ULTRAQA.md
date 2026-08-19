# ULTRAQA — Analyze-76

**Verdict:** PASS (local workspace)  
**Date:** 2026-08-19

## Goal
Canonical Artifact signatures; mutation fails verify; C0/C1 green.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | crypto | mutate type/provenance/policy/hash | verify err | `canonical_verify_fails_when_artifact_fields_change` | PASS |
| U2 | store | payload vs claimed hash | HashMismatch | `cas_publish_resolve_and_hash_mismatch` | PASS |
| U3 | C1 | private artifact after re-sign | AccessDenied | `security_baseline_passes` | PASS |
| U4 | Out | Event still canonical from #40 | unchanged | event tests | PASS |
| U5 | gate | workspace test + clippy `-D warnings` | green | exit 0 | PASS |
