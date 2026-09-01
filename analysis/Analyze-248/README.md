# Analyze-248 — Plane register (QUEUE #213)

## Done
- `OperationalPlane` registers `ExecutionLlmCsu` with `MockBackend` (same factory/activate path as other execution CSUs)
- Fan-out skip: execution-basic ignores `text.generate.local`; execution-llm ignores math/echo/uppercase (generate-local schema + wrong action still fail-closed)
- `submit_problem` non-math → `SubmitOutcome::Executed` + MockBackend text; no fake VERIFIED
- Named tests: `non_math_prompt_completes_via_execution_llm_mock`; `calculate_two_plus_two_stays_execution_basic`; C1 `c1.pipeline.calculate_2_plus_2`
- RFC-D [`AIRA-RFC-0108`](../../specs/rfc/AIRA-RFC-0108-plane-register-execution-llm.md)
- QUEUE `#213` **DONE**; first OPEN `#214`
- RFC-0104 remains file-free (`phase_k_rfc_0104_id_free`)

## Out
Activate policy (`#214`); process backend (`#215`); Desktop Work generate; RFC-0104 (`#216`); Core inference host; Reduction catalog change; CSU→CSU Cargo dep on inventory/acquisition.
