# AIRA-RFC-0241 — Desktop Settings Ollama process bind

## 1. Summary

Desktop Settings may bind the staff submit executor to host **Ollama** (`AIRA_LLM_BACKEND=process`) and select a model from `ollama list`. Applied only after node restart. **Executed ≠ VERIFIED.** Process ≠ mock. No Phase D activate forgery.

## 2. Problem Statement

Developer Preview Desktop spawned `aira-node` without LLM env, so submit always used reference MockBackend while System showed `executor: mock`. Operators could set env by hand; Settings → Models catalog is AIRA inventory, not `ollama list`.

## 3. Motivation

Product need: bind a real local CLI LLM from Settings without claiming VERIFIED or inventing Pack 3 QUEUE atoms.

## 4. Scope

- Optional settings fields: `llm_backend`, `llm_process_bin`, `llm_ollama_model` on `aira:schema:desktop:settings:0.1`
- `apply_node_llm_env` on `aira-node` spawn; pidfile records applied LLM facts
- Host `ollama list` probe + Settings UI bind/select
- Honest executor display from applied/configured settings (not GUI process env)

## 5. Non-Goals

```text
Fake VERIFIED / AlwaysActivated for ollama
GPU marketplace / auto-download from Settings
expected_model_ref auto-sync with activate tip (still #339 / RFC-0223 when set)
Pack 3–7 QUEUE invention
Changing C1 math path
```

## 6. Behavior

| Action | Effect |
|--------|--------|
| Select mock | `llm_backend=mock`; node env forces mock after restart |
| Select process + model from `ollama list` | `llm_backend=process`, `AIRA_LLM_PROCESS_ARGS=run <model>` after restart |
| Generate without Phase D activate | fail-closed CapsuleFailed (unchanged) |
| Generate with process | status **executed**, never silent VERIFIED |

RestartNeeded when saved LLM fields ≠ pidfile-applied facts.

## 7. Acceptance

1. Schema validate accepts optional LLM fields; process without model fail-closed on normalize.
2. Fresh start writes LLM into pidfile; attach reports pidfile LLM (not later disk edits).
3. Settings UI lists `ollama list` names; bind persists; executor label becomes `process` after restart.
4. Docs/help state process ≠ mock and executed ≠ VERIFIED.

## 8. References

- RFC-0024 (settings schema), RFC-0110 (process backend), RFC-0204 (executor honesty), RFC-0223 (binding)
- `docs/desktop-ux.md`, `docs/help/*/model.select.md`
