# Analyze-394 — Brief

## Goal

Settings → Models: bind host Ollama as process executor; choose model from `ollama list`; after node restart System/Work show `executor: process` honestly; never forge VERIFIED.

## Done

- Schema + `DesktopSettings` LLM fields + normalize
- `apply_node_llm_env` on spawn + pidfile LLM facts
- `ollama list` probe + Settings UI
- Executor display from applied/settings (not GUI env)
- **RFC-0242:** host LLM required for Work; OP-001/C1 math → legacy; C1 process smoke
- **RFC-0243:** per-request `host_cli_model` argv; lifecycle/attach honesty; Verify after quarantine; async catalog; timeouts; Work result binding; M6 ≠ installed-product
- **Re-audit P0 (ef0503a):** R1 Work readiness uses GUI/`DesktopPaths` settings (no load-or-create under data_root); R2 `host_cli_model` from signed evidence; R3 file weights fail-closed on ollama ProcessBackend
- **Re-audit P1:** R4 admitted `model_ref` forces `text.generate.local` (no math/echo/upper escape); R5 bounded binder read; R6 exclusive unique materialize temps + concurrent test
- **Re-audit P2:** bidirectional Work↔catalog lock; async Add; `ollama list` kill+wait; Stop confirms death before pidfile clear; Compare A via `WorkJobEvent::ComparePrimary`; timeout GUI + pidfile + `work.timed_out`
- **Re-audit P3:** one Settings→Models catalog by source (host Ollama / local file); Select host-ollama makes default tip or says it did not; artifact path in Technical details; primary chrome without Phase/tip jargon; help EN/UK matches the host-LLM gate

## Canon

- Work readiness fail-closes on mock / missing `llm_ollama_model`
- Math text is not a model-free escape
- C1/alpha/C0 causality use process `Executed` ≠ VERIFIED
- Settings tip ≠ sole executor; admission binding drives per-request ollama model
- Executed ≠ VERIFIED; Choice ≠ VERIFIED; activated ≠ weight-verified
- Catalog mutate and Work submit are mutually exclusive; Cancel honesty (no Esc-as-cancel)
