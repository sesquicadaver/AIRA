# GUI audit fix plan (4af7585)

Source: `p-only/AIRA-GUI-audit-4af7585-2026-09-19.md` and `…-add.md`.
Order is sequential. Do not start a later pack while an earlier one is open.
Out of scope: a new LLM provider, installing Ollama, the Ollama HTTP API, replacing egui.

Acceptance for the whole plan: a user configures a supported model, runs a prompt, reads the answer, and fixes a typical error without `model_ref`, activation tip, or RFC.

## P0 — Truthful readiness

Status: **done** in this patch (`running_mock_blocks_saved_process`, `local_file_is_not_ready_on_ollama_executor`, `ollama_prefix_without_host_cli_does_not_fall_back_to_argv`).

- A running node: Work readiness and Run use the applied executor (pidfile / applied), not settings that were only saved. A stopped node: the next-start configuration.
- Saved ≠ applied for backend (Mock ↔ Process), binary, or timeout → one action, restart. Switching host model A→B on an already applied Process does not require a restart.
- A local file on the Ollama executor is not Work ready. Text: the file is prepared; this executor cannot run it.
- `ProcessBackend`: the prefix `aira:model:ollama-` is not proof of type. Missing `host_cli_model` blocks generate. No argv fallback.

Done when: Mock→Process without restart does not admit Work; a prepared file is not ready; an ollama-prefix ref without `host_cli_model` does not run tip argv.

## P1 — One catalog

Status: **done** in this patch (`projection_file_source_omits_ollama_rows`, `use_ollama_waits_for_empty_list`).

- Settings and Work share one projection: name, source, availability. `model_ref` and hash stay in details.
- Choosing a model for a request does not change the default tip. Make-default is a separate action.
- Ollama list has an explicit loading state. “Use Ollama” binds only an exact pick or exact saved name — never the first list row (`#364`).
- The local-file source does not list Ollama rows or offer a Prepare that is designed to fail.
- The verification document path sits in the add-file step, not inside closed technical details.

## P2 — Screens and copy

Status: **done** in this patch (Work selector, System conclusions, Settings group order, help start without a model-free math promise).

- Work: selector (“Default: name”), prompt, Run, answer using free height, model that actually ran, one verification status, Copy. Compare opens a second selector, not a second full catalog.
- System: three short conclusions and the next action. Journal, PID, addresses stay collapsed.
- Settings: Models, General, Connection. Timeout and operator network fields under Advanced. Do not repeat Saved and Applied when they match.
- Errors next to the action; codes under Details. Remaining English operator strings go through i18n.
- F1 Start no longer promises math without a model. RFC, OP-001, and legacy leave the product screens.

## P3 — Job and timeout

Status: **done**.

- Button, shortcut, and handler share `work_run_available`. “Work: ready” only when P0 readiness allows the run; otherwise the strip says not ready.
- Node Stop is not presented as cancel. The word Cancel appears only after `generate process timed out` (child already killed).
- Timeout is shown in seconds. `0` is rejected (not dropped). HTTP read waits 15s past the process deadline. A transport read error stays a transport error.

## P4 — Window check

Status: **done**. Contrast and typography stay a later pass. This pass did not take screenshots.

- F1 docks at 320 px only when the window is at least 880 px, so 900 and 1600 leave a central column of at least 560. At 560, F1 is a floating window and Work keeps the full width (`window_check_f1_keeps_central_column`).
- Tabs, the status strip, the model selector, and the “model that ran” line wrap, so a long name does not require one unbreakable row.
- The scenario truths from P0–P3 stay in force: no model is not ready; a request pick does not change the default; Mock→Process needs a restart; a local file is not ready on Ollama; a failed Compare B keeps A; timeout is seconds and not a malformed HTTP error.

Follow-on, not part of this audit: [`phase-y-plan.md`](phase-y-plan.md) `#359`–`#375`. Metadata HTTP starts at `#368` only.
