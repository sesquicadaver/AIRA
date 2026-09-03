# Analyze-263 — missing sandbox fail-closed (QUEUE #228)

## Done
- Opt-in `ProcessBackend::with_sandbox_required()` / `AIRA_LLM_SANDBOX_REQUIRED`
- Missing Landlock ABI / non-Linux → `SANDBOX_REQUIRED` CapsuleFailed (never unsandboxed success)
- ollama-style + required → `SANDBOX_REQUIRED_LOOPBACK`
- When the host can isolate: apply netns + Landlock + seccomp
- RFC [`AIRA-RFC-0121`](../../specs/rfc/AIRA-RFC-0121-sandbox-required.md)
- Living `phase_m_sandbox_required_228`; QUEUE `#228` **DONE**; first OPEN `#229`
- C1 2+2 unchanged; RFC-0117 file-free

## Out
OS vs AIRA-mediated schema (`#229`); RFC-0117 body; Core inference; GPU marketplace; forcing ollama through netns; requiring ollama in CI.
