# AIRA-RFC-0214 — E2E fail-closed Pack 1 subset (RFC-D)

## 1. Summary

QUEUE `#329` (Phase V / Repair Pack 1): named end-to-end fail-closed suite for the three Pack 1 acceptance scenarios — same text + different model must not incorrect-reuse; Settings mid-run must not mutate the admitted snapshot; verify-file tampered before activate must reject with no activated pointer. Glue asserts C1 VERIFIED and generate-local executor facts.

## 2. Problem Statement

Unit coverage for `#324`–`#328` existed in-crate, but Pack 1 §6 / `aira-repair.md` named an e2e subset that was not a single living contract. Without it, cross-path regressions could land as green unit tests with a broken accept matrix.

## 3. Motivation

`aira-repair.md` Pack 1 §6; [`docs/phase-v-plan.md`](../../docs/phase-v-plan.md); parent consolidating RFC-0208 (file-free until `#330`; now DONE).

## 4. Scope

- Integration test `crates/aira-flow/tests/phase_v_pack1_e2e.rs`
- Scenarios: reuse/model, settings-during-run, verify-tamper-before-activate
- Glue: C1 `Calculate 2 + 2` VERIFIED + generate-local stamps `model_ref` / `model_content_hash`
- Living smoke: `phase_v_doc` requires RFC-0214 + Analyze-366 + tip `#330`

## 5. Non-Goals

```text
Pack 2 multi-model GUI / «Порівняти»
Full repair §6 (replicas, share, quota, GUI)
RFC-0208 consolidating close (#330)
Minting VERIFIED for generate-local
GPU marketplace / LLM-in-Core
```

## 6. Compatibility

No runtime API change. Tests exercise existing admit/reuse/activate/generate-local paths.

## 7. Security / Integrity

Fail-closed assertions only: wrong-model reuse must not Success-cache; admitted snapshot identity survives Settings-like mutation of a local copy; tampered verified weights → `ActivateHashMismatch` and must not write `activated.latest.json`.

## 8. Acceptance

```text
pack1_e2e_same_text_different_model_no_reuse green.
pack1_e2e_settings_mid_run_keeps_admission_binding green.
pack1_e2e_verify_tamper_before_activate_rejected green.
pack1_e2e_c1_and_generate_local_facts_hold green.
C1 Calculate 2 + 2 unchanged.
Tip advances to first OPEN #330.
```

## 9. References

- QUEUE `#329` · Analyze-366
- Depends on RFC-0209…0213
- Parent consolidating: RFC-0208 (file-free until `#330`)
