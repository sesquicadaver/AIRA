# ULTRAQA — Analyze-75

**Verdict:** PASS (local workspace)  
**Date:** 2026-08-19

## Goal
Canonical Event signatures; mutation fails verify; C0/C1 still green.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | crypto | mutate event_type / refs / payload_hash | verify err | `canonical_verify_fails_when_event_type_or_causal_refs_change` | PASS |
| U2 | log | secret payload after re-sign | SecretMaterial | `secret_material_rejected_in_payload_ref` | PASS |
| U3 | C1 | `sec.no_secrets_in_events` | pass | `tests::security_baseline_passes` | PASS |
| U4 | contract | no event-path domain fallback | absent in log/invariants | grep | PASS |
| U5 | Out | Artifact still content_hash | unchanged | `make_artifact` | PASS |
| U6 | gate | `cargo test --workspace --locked` + clippy `-D warnings` | green | exit 0 | PASS |
