# Living spec — Analyze-396

| ID | Requirement | Module | Test | Status |
|----|-------------|--------|------|--------|
| LS396-001 | Two real Ollama models, no restart | `ProcessBackend` `host_cli_model` | `installed_product_two_real_ollama_models` | **DONE** (opt-in; this host) |
| LS396-002 | File weights fail-closed on ollama | `HOST_OLLAMA_BINDING_REQUIRED` | same | **DONE** |
| LS396-003 | Timeout ≠ VERIFIED | `TIMED_OUT` | same | **DONE** |
| LS396-004 | GUI 560/900/1600, Compare fail-leg, failed Stop | Desktop | — | **ABSENT** (not executed) |
| LS396-005 | Default CI has no `ollama run` | `.github/workflows/ci.yml` | `installed_product_opt_in_is_not_default_ci` | **DONE** |
