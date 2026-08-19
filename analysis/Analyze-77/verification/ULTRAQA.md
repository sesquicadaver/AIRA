# ULTRAQA — Analyze-77

**Verdict:** PASS (local workspace)  
**Date:** 2026-08-19

## Goal
Canonical Object signatures; mutation fails verify; C0/C1 green.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | crypto | mutate object fields | verify err | `canonical_verify_fails_when_object_fields_change` | PASS |
| U2 | store | unsigned / mutated create | Unsigned / InvalidSignature | `create_rejects_unsigned_and_mutated_object_signature` | PASS |
| U3 | flow | node identity submit | `verify_canonical` | `local_session_submit_signs_with_node_identity` | PASS |
| U4 | Out | Artifact/Event still canonical | unchanged | artifact/event tests | PASS |
| U5 | gate | workspace test + clippy `-D warnings` | green | exit 0 | PASS |
