# Analyze-394 — Brief

## Goal

Settings → Models: bind host Ollama as process executor; choose model from `ollama list`; after node restart System/Work show `executor: process` honestly; never forge VERIFIED.

## Done

- Schema + `DesktopSettings` LLM fields + normalize
- `apply_node_llm_env` on spawn + pidfile LLM facts
- `ollama list` probe + Settings UI
- Executor display from applied/settings (not GUI env)
- **RFC-0242:** host LLM required for Work; OP-001/C1 math → legacy; C1 process smoke

## Canon

- Work readiness fail-closes on mock / missing `llm_ollama_model`
- Math text is not a model-free escape
- C1/alpha/C0 causality use process `Executed` ≠ VERIFIED
