# Living spec — Analyze-397 (`#388`)

| ID | Requirement | Module | Test / evidence | Status |
|----|-------------|--------|-----------------|--------|
| LS397-001 | Concrete git SHA in evidence | `run.json` `git_sha` | Analyze-397 | **DONE** |
| LS397-002 | Ollama model digests in evidence | `run.json` `ollama_digests` | Analyze-397 | **DONE** |
| LS397-003 | Two real Ollama models + negatives | `ProcessBackend` | `installed_product_two_real_ollama_models` | **DONE** (opt-in) |
| LS397-004 | CLI+HTTP two-model staff path at SHA | `phase_x_m6_acceptance` | M6 fixtures | **DONE** |
| LS397-005 | GUI two-model readiness at SHA | `phase_x_m6_gui` | M6 fixtures | **DONE** |
| LS397-006 | Analyze-396 ≠ HEAD proof | docs + `not_analyze_396_head_proof` | `#388` | **DONE** |
| LS397-007 | Real-Ollama GUI click-through | Desktop | — | **ABSENT** (manual) |
