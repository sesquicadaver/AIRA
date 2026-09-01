# Analyze-250 — Process backend (QUEUE #215)

## Done
- `ProcessBackend` in `csu/execution-llm/src/process.rs`: PATH or explicit path; fixed argv via `Command::new`; never `sh -c`
- Missing binary / spawn fail / non-zero / timeout / empty stdout → `CapsuleFailed` (`missing_process_binary_is_capsule_failed`)
- Activate gate still runs before spawn (`missing_process_binary_does_not_skip_activate_gate`)
- OperationalPlane default and CI keep `MockBackend` (`default_plane_keeps_mock_backend`; `with_mock_backend` on `open`)
- Opt-in: `ExecutionLlmCsu::with_process_backend` / `OperationalPlane::bind_process_backend`; env `AIRA_LLM_BACKEND=mock|process` (default mock)
- Network: AIRA opens no sockets; ollama child may use loopback; llama.cpp-style argv is offline; no WAN
- C1 `Calculate 2 + 2` unchanged (`math.eval.safe` / execution-basic)
- RFC-D [`AIRA-RFC-0110`](../../specs/rfc/AIRA-RFC-0110-process-backend.md)
- QUEUE `#215` **DONE**; first OPEN `#216`
- RFC-0104 remains file-free (`phase_k_rfc_0104_id_free`)

## Out
Desktop Work generate + consolidating RFC-0104 (`#216`); Core inference host; Reduction catalog change; CSU→CSU Cargo dep on inventory/acquisition; bundling model weights; GPU marketplace; requiring ollama/llama.cpp in CI.
