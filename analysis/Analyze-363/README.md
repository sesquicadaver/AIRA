# Analyze-363 — Reuse after constraints

**QUEUE:** `#326` · **RFC-0211**

## Outcome

Reuse-index keys are admission-scoped via `AdmissionSnapshot::reuse_catalog_key`. `RequireNewExecution` skips lookup/record. Same text with different `model_ref` does not hit text-only reuse.
