# AIRA-RFC-0240 — Two-model installed acceptance M6 (RFC-D)

## 1. Summary

QUEUE `#357` (Phase X / Pack 2 X3): living installed-product acceptance that two real local models share one staff path — GUI catalog/Work readiness, CLI `--model-ref` / HTTP `admission.model_ref` (same `AdmissionConstraints`), cold restart keeps tip+slots, and Required-unavailable never silent-substitutes.

## 2. Problem Statement

M1–M5 (+ X2 `#348`–`#356`) covered unit/API surfaces. Pack 2 / audit M6 required a single green e2e contract across GUI+CLI+HTTP without merging into consolidating close (`#358`).

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X3 after apply-diff honesty (#356). Parent RFC-0226 reserved until `#358`.

## 4. Scope

- `crates/aira-flow/tests/phase_x_m6_acceptance.rs` — CLI/HTTP admission + cold restart + fail-closed
- `crates/aira-desktop-runtime/tests/phase_x_m6_gui.rs` — Settings prepare/select + Work readiness/Compare
- Tip → first OPEN `#358`
- Living smoke: `phase_x_doc` requires RFC-0240 + Analyze-394 + M6 test names

## 5. Non-Goals

```text
RFC-0226 consolidating close (#358)
Pack 3 ratings / Pack 4–5 share
Minting VERIFIED for generate-local / mock used-model stamp (#339)
Full egui pixel e2e / marketplace download-delete
```

## 6. Compatibility

No new runtime API. Harness reuses activate/select/catalog/work_readiness/`LocalSession` paths from `#344`–`#354`.

## 7. Security / Integrity

Fail-closed only: Required missing/unready → explained select error and no Executed Success with a substitute model; Compare missing leg → not ready.

## 8. Acceptance

```text
m6_e2e_two_models_cli_http_admission_paths green.
m6_e2e_cold_restart_preserves_tip_and_slots green.
m6_e2e_unavailable_required_fail_closed_no_substitute green.
m6_e2e_gui_catalog_and_work_readiness_two_models green.
Tip → first OPEN #358; RFC-0240.
```

## 9. References

- QUEUE `#357` · Analyze-394
- Depends on RFC-0227…0239
- Parent consolidating: RFC-0226 (file-free until `#358`)
