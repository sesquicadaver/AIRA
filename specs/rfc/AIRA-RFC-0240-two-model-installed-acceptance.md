# AIRA-RFC-0240 — Two-model installed acceptance M6 (RFC-D)

## 1. Summary

QUEUE `#357` (Phase X / Pack 2 X3): living **unit/e2e harness** that two local models share one staff path — GUI catalog/Work readiness, CLI `--model-ref` / HTTP `admission.model_ref` (same `AdmissionConstraints`), cold restart keeps tip+slots, and Required-unavailable never silent-substitutes.

**Honesty (RFC-0243 Pack G):** M6 tests use **fixture weight bytes** (`m6-*-weights`), not two real host Ollama models. CI green on M6 ≠ **installed-product** acceptance complete. Installed-product gate is opt-in / manual / nightly with two real `ollama list` models — see §8b and [`docs/installed-product-llm-acceptance.md`](../../docs/installed-product-llm-acceptance.md).

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
Claiming fixture-M6 as installed-product complete
```

## 6. Compatibility

No new runtime API. Harness reuses activate/select/catalog/work_readiness/`LocalSession` paths from `#344`–`#354`.

## 7. Security / Integrity

Fail-closed only: Required missing/unready → explained select error and no Executed Success with a substitute model; Compare missing leg → not ready.

## 8. Acceptance (M6 fixture harness)

```text
m6_e2e_two_models_cli_http_admission_paths green.
m6_e2e_cold_restart_preserves_tip_and_slots green.
m6_e2e_unavailable_required_fail_closed_no_substitute green.
m6_e2e_gui_catalog_and_work_readiness_two_models green.
Tip → first OPEN #358; RFC-0240.
```

## 8b. Installed-product acceptance (opt-in; not CI default)

M6 §8 is fixture weights. This section is the real-host gate. Default CI does not run it. Opt-in process evidence: `installed_product_two_real_ollama_models` (`#[ignore]`) via [`scripts/installed-product-llm-acceptance.sh`](../../scripts/installed-product-llm-acceptance.sh). A green fixture harness is not installed-product complete. GUI widths 560/900/1600 stay manual.

**`#383` honesty:** a local installed file may appear in the catalog after Prepare; that is **not** the same criterion as Work ready for generate on the Ollama executor. File weights must fail-closed on ollama-style ProcessBackend (R3). No new file-executor is introduced by this gate.

**`#388`:** SHA-pinned evidence lives under [`analysis/Analyze-397/`](../../analysis/Analyze-397/) (`git_sha` + Ollama digests + process real models + M6 fixture GUI/CLI/HTTP). Analyze-396 is **not** proof of HEAD.

Reproducible manual/nightly gate (document evidence in Analyze notes):

1. Two real models from `ollama list` bound via Settings; Required A then B **without** node restart (RFC-0243).
2. Compare A|B with distinct models; fail one leg → A retained.
3. File path: Add → Verify (artifact) → Prepare. The file stays **visible** in the Local file catalog source. On the host Ollama ProcessBackend it is **not** Work-ready for generate (`#383` / R3) — prepared ≠ executable via `ollama run`. Do not add a file-executor only to make this row green.
4. Timeout / cancel honesty; failed Stop leaves window + pidfile.
5. Layout smoke at widths 560 / 900 / 1600.

## 9. References

- QUEUE `#357` · Analyze-394 · RFC-0243
- Depends on RFC-0227…0239
- Parent consolidating: RFC-0226 (file-free until `#358`)
