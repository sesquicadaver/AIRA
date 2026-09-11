# Analyze-366 — E2E fail-closed tests

**QUEUE:** `#329` · **RFC-0214**

## Outcome

Pack 1 §6 fail-closed subset lives in `crates/aira-flow/tests/phase_v_pack1_e2e.rs`: wrong-model reuse blocked, admitted snapshot frozen vs Settings-like mutation, verify-tamper-before-activate rejected, plus C1 + generate-local executor-facts glue.
