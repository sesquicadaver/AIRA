# AIRA-RFC-0110 — Generate-local process CLI backend (RFC-D)

## 1. Summary

Phase K `#215`: `execution-llm` gains a [`ProcessBackend`](../../csu/execution-llm/src/process.rs) that runs a host-local CLI (ollama and/or llama.cpp-style **fixed argv**) behind [`GenerateBackend`](../../csu/execution-llm/src/lib.rs). Missing binary, spawn failure, non-zero exit, timeout, or empty stdout → `CapsuleFailed`, never a fake VERIFIED result. The reference plane and CI keep [`MockBackend`](../../csu/execution-llm/src/lib.rs). RFC-0104 stays file-free until `#216`.

## 2. Problem Statement

After `#214`, generate-local can complete via MockBackend once Phase D activate is present, but there is no real local process adapter. Operators cannot bind ollama/llama.cpp, and a missing CLI must not look like CapsuleCompleted / VERIFIED.

## 3. Motivation

[`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K6 and QUEUE `#215`: argv/local CLI adapter; missing binary fail-closed; tests keep mock. Book I §2: inference stays in the Execution CSU, not Core. RFC-0105 freezes `network=none` and `shell=false`.

## 4. Scope

- `ProcessBackend` in `csu/execution-llm` (not `aira-core`)
- PATH or explicit filesystem path lookup (not a marketplace)
- Fixed argv via `Command::new` (never `sh -c`)
- Activate gate from `#214` still runs **before** spawn
- Env `AIRA_LLM_BACKEND=mock|process` (default mock)
- Named tests: missing binary → CapsuleFailed; MockBackend still the plane/CI path
- RFC-0104 stays file-free until `#216`

## 5. Non-Goals

```text
Desktop Work generate path / consolidating RFC-0104 (#216)
Cargo dep execution-llm → inventory/acquisition (CSU ↛ CSU)
llama/ggml in aira-core
bundling model weights
GPU marketplace / LLM in Core / blockchain
changing Reduction catalog
requiring ollama/llama.cpp on GitHub Actions
WAN / remote model fetch
```

## 6. Current Behavior

Only MockBackend is selectable. No process spawn. Missing CLI cannot fail closed because there is no process path.

## 7. Proposed Change

```text
execution-llm:
  ProcessBackend { program, fixed args, timeout }
  resolve program via PATH or explicit path
  missing / spawn fail / non-zero / timeout / empty stdout → CapsuleFailed
  prompt appended as last argv token (not a shell string)
  activate gate still precedes generate()

OperationalPlane:
  default open() + bind_activate_gate() stay MockBackend
  bind_process_backend() is opt-in
  LocalSession does not construct ProcessBackend

Env:
  AIRA_LLM_BACKEND=mock|process  (default mock)
  AIRA_LLM_PROCESS_BIN           (program name or path)
  AIRA_LLM_PROCESS_ARGS          (whitespace-split extra argv; not sh -c)
  AIRA_LLM_PROCESS_TIMEOUT_MS    (default 30000)

C1 Calculate 2 + 2:
  unchanged (execution-basic / math.eval.safe)
```

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend = CSU; no Core inference
- Schema: none (RFC-0105 `network=none` / `shell=false` unchanged)
- Tests: `missing_process_binary_is_capsule_failed`; `backend_from_env_defaults_to_mock_not_process`; `default_plane_keeps_mock_backend`; `missing_process_binary_on_plane_is_capsule_failed`; `mock_backend_completes_valid_generate_local`; `calculate_two_plus_two_stays_execution_basic`; `phase_k_process_backend_215`
- Canonical `ArtifactType`: none

## 9. Compatibility Impact

Additive. Default plane/CI behavior is unchanged (MockBackend). Operators opt in via `bind_process_backend` or `AIRA_LLM_BACKEND=process`.

## 10. Security Impact

- `shell=false`: spawn is `Command::new(program)` + explicit args. Never `sh -c`.
- `network=none` on the payload remains required. ProcessBackend opens **no sockets**.
- Network choice: AIRA does not initiate WAN. A child such as `ollama` may use **loopback** to a local daemon; llama.cpp-style argv is offline. No WAN from this crate.
- Missing binary is fail-closed, not a fake VERIFIED / CapsuleCompleted.

## 11. Privacy Impact

Process stdout becomes the ExecutionArtifact `result`. Prompts are passed as argv to a local child. No user traffic leaves the host via AIRA.

## 12. Policy Impact

Activate gate (`#214` / RFC-0109) still applies before spawn. Acquisition/download policy unchanged.

## 13. Failure Semantics

Missing binary / spawn failure / non-zero / timeout / empty stdout MUST emit `CapsuleFailed`. MUST NOT emit `VerifiedResultArtifact`. On the plane, evidence-basic still emits `FailureEvidenceCreated` for CapsuleFailed.

## 14. Rollback Plan

Revert `process.rs`, plane `bind_process_backend`, this RFC, Analyze-250, and living-spec `#215` rows. Keep MockBackend. Do not create RFC-0104.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm
cargo test -p aira-flow --lib
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_k_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / `execution-basic`. CI MUST NOT require ollama or llama.cpp.

## 16. Migration Plan

None. Default remains mock. Operators set `AIRA_LLM_BACKEND=process` and `AIRA_LLM_PROCESS_BIN` (or call `bind_process_backend`) after a Phase D activate.

## 17. Alternatives Considered

- Default-bind ProcessBackend on the reference plane — rejected; CI must not require a local LLM CLI.
- `sh -c` wrapper for flexible CLI strings — rejected (`shell=false`).
- HTTP client to ollama on localhost from this crate — rejected for this atom; argv adapter only. Loopback is a child concern, documented, not WAN.
- Put process spawn in `aira-core` — forbidden (anti-mission).
- Cargo-dep on inventory/acquisition — rejected (CSU ↛ CSU).
- File this as RFC-0104 — rejected; RFC-0104 is reserved for `#216`.

## 18. Evidence

- Book I §2: LLM Backend is a CSU, not Core.
- RFC-0105 generate-local constraints; RFC-0106 MockBackend; RFC-0109 activate gate.
- [`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K6; QUEUE `#215`.

## 19. Open Questions

Desktop Work generate UX (`#216` / RFC-0104) — out of this RFC.
