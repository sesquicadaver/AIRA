# Analyze-246 — `execution-llm` CSU + mock (QUEUE #211)

## Done
- Workspace crate `csu/execution-llm` (`aira-csu-execution-llm`)
- `ExecutionLlmCsu` + `GenerateBackend` + `MockBackend` (deterministic; no shell/network)
- Fail-closed when no backend is bound; strict generate-local parse (`deny_unknown_fields`)
- Named tests: `mock_backend_completes_valid_generate_local`; `missing_backend_is_capsule_failed`; wrong action / extra properties / missing prompt
- RFC-D [`AIRA-RFC-0106`](../../specs/rfc/AIRA-RFC-0106-execution-llm-mock.md)
- QUEUE `#211` **DONE**; first OPEN `#212`
- RFC-0104 remains file-free (`phase_k_rfc_0104_id_free`)
- `#214` activate gate left as TODO/fail-closed placeholder only

## Out
Reduction bind (`#212`); plane register (`#213`); activate policy (`#214`); process backend (`#215`); Desktop Work generate; RFC-0104 (`#216`); Core inference host; Cargo dep on inventory/acquisition CSUs.
