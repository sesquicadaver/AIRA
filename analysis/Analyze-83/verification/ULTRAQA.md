# ULTRAQA — Analyze-83

**Verdict:** PASS (local)  
**Date:** 2026-08-19

## Goal
Tenant behavior unchanged after file split.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | isolation | cross-CSU publisher blocked | same | `tenant_isolation_blocks_cross_csu_publisher` | PASS |
| U2 | persist | save/load / meta mismatch | same | `save_load_survives_reset` / `meta_pubkey_mismatch_fails_closed` | PASS |
| U3 | ceremony | rotate + revoke + audit | same | `rotate_happy_path_and_audit` / `revoke_removes_dir_map_and_audits` | PASS |
| U4 | prune | keep/latest/orphan | same | prune_* tests | PASS |
| U5 | clippy | workspace -D warnings | green | cargo clippy --workspace | PASS |
| U6 | workspace | cargo test --workspace | green | local | PASS |
