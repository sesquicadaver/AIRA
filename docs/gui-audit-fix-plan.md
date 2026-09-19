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

Status: **open**.

- Settings and Work share one projection: name, source, availability. `model_ref` and hash stay in details.
- Choosing a model for a request does not change the default tip. Make-default is a separate action.
- Ollama list has an explicit loading state. The first “Use Ollama” finishes the bind after the list returns.
- The local-file source does not list Ollama rows or offer a Prepare that is designed to fail.
- The verification document path sits in the add-file step, not inside closed technical details.

## P2 — Screens and copy

Status: **open**.

- Work: selector (“Default: name”), prompt, Run, answer using free height, model that actually ran, one verification status, Copy. Compare opens a second selector, not a second full catalog.
- System: three short conclusions and the next action. Journal, PID, addresses stay collapsed.
- Settings: Models, General, Connection. Timeout and operator network fields under Advanced. Do not repeat Saved and Applied when they match.
- Errors next to the action; codes under Details. Remaining English operator strings go through i18n.
- F1 Start no longer promises math without a model. RFC, OP-001, and legacy leave the product screens.

## P3 — Job and timeout

Status: **open**.

- Button, shortcut, and handler share one availability check. “Work: ready” only when P0 allows the run.
- Node Stop is not presented as cancel. Cancel appears only after the process is confirmed dead.
- Timeout is shown in seconds, with slack for the HTTP response. `0` is not silently ignored. A transport error is not reported as malformed HTTP.

## P4 — Window check

Status: **open**. After P0–P3, not instead of them.

First launch with no model; A/B without changing the default; Mock→Process; incompatible file; failed B keeps A; timeout; long names; widths 560 / 900 / 1600 with F1 open. Typography and contrast are a later pass, after the scenario is truthful.
