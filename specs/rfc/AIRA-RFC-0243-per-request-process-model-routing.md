# AIRA-RFC-0243 — Per-request ProcessBackend model routing

## 1. Summary

Desktop Work Required / Compare must run the **admission-selected** host Ollama (or weight) model on each submit without restarting `aira-node`. Settings Ollama bind remains the **default tip / cold-start** argv only. **Executed ≠ VERIFIED.** Choice ≠ VERIFIED.

## 2. Problem Statement

Spawn fixed `AIRA_LLM_PROCESS_ARGS` + `AIRA_LLM_EXPECTED_MODEL_REF` locked every Work submit to the Settings tip. Compare A|B and Required switches could not change the executed host CLI model without restart (audit GUI/LLM #1).

## 3. Motivation

Operators need two installed Ollama models on one node session: Required A then B, and Compare A|B, with fail-closed mismatch (no silent substitute).

## 4. Scope

- Activate binding carries optional `host_cli_model` (pointer / bind-marker) into `ExecutorFacts.host_cli_model`
- `ProcessBackend::generate` rewrites ollama-style argv to `run <host_cli_model>` when binding provides it; spawn `EXPECTED_MODEL_REF` is advisory in that case
- `apply_node_llm_env` sets process bin + default tip args; does **not** lock `AIRA_LLM_EXPECTED_MODEL_REF` to tip
- Collision-resistant `host_ollama_model_ref` (hash suffix) so `org/model:latest` ≠ `org_model:latest`
- Lifecycle: activated slot ≠ `verified=true` for host-ollama-only binds
- Attach: applied LLM facts from pidfile; disk ≠ pidfile → RestartNeeded
- Optional `llm_process_timeout_ms` → `AIRA_LLM_PROCESS_TIMEOUT_MS`; HTTP submit timeout aligned

## 5. Non-Goals

```text
Ollama HTTP Generate API (wave 2+)
GPU marketplace / egui replacement
Minting VERIFIED for process generate
Pack 3–7 QUEUE invention
```

## 6. Behavior

| Surface | Rule |
|---------|------|
| Settings tip | Default cold-start `run <tip>`; not sole executor for all Work choices |
| Admission model_ref | Selects activate slot; host_cli_model from that slot drives argv |
| Compare A then B | Each leg uses its own host_cli_model; B fail does not rewrite A |
| Expected mismatch | Without host_cli_model, spawn expected vs binding still CapsuleFailed |
| Catalog host-ollama | Select/Prepare do not call `activate_verified`; Select skips re-copy when tip matches |

## 7. Acceptance

1. Unit: generate with host_cli_model A then B (same ProcessBackend tip) → argv uses A then B.
2. Unit: host-ollama lifecycle entry `verified=false` unless weight-verified pointer exists.
3. Unit: `host_ollama_model_ref("org/model:latest")` ≠ `host_ollama_model_ref("org_model:latest")`.
4. Unit: attach applied LLM = pidfile; saved tip differ → RestartNeeded.
5. Docs: Executed ≠ VERIFIED; M6 fixture acceptance ≠ installed-product gate (see RFC-0240 honesty).

## 8. References

- RFC-0241 (Settings Ollama bind), RFC-0242 (host LLM required), RFC-0237 (Compare)
- Audit GUI/LLM 77dd97e findings #1 #2 #5 #7 #9
- Analyze-394 living matrix
