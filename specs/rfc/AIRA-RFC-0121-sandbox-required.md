# AIRA-RFC-0121 — Sandbox-required fail-closed when kernel/OS missing (RFC-D)

## 1. Summary

Phase M `#228`: operator policy `ProcessBackend::with_sandbox_required()` / `AIRA_LLM_SANDBOX_REQUIRED=1|true|yes`. When OS sandbox is required, generate MUST apply Landlock + seccomp + netns. Missing Landlock ABI, non-Linux OS, or ollama-style host loopback is **fail-closed** (`SANDBOX_REQUIRED` / `SANDBOX_REQUIRED_LOOPBACK`) — never unsandboxed success or fake VERIFIED. Parent probes Landlock ABI without restricting the parent. RFC-0117 stays file-free until `#230`.

## 2. Problem Statement

`#225`–`#227` fail-close when a **requested** restrict cannot be applied. Default remains unsandboxed. An operator who **requires** OS isolation had no switch: generate on macOS/Windows or a kernel without Landlock could still complete.

## 3. Motivation

Invariant: sandbox required but unavailable → CapsuleFailed. CI keeps MockBackend. Opt-in flags stay for partial sandbox without the required policy.

## 4. Scope

- [`csu/execution-llm/src/sandbox.rs`](../../csu/execution-llm/src/sandbox.rs) + `with_sandbox_required` in [`process.rs`](../../csu/execution-llm/src/process.rs)
- Parent Landlock ABI probe [`landlock::kernel_available`](../../csu/execution-llm/src/landlock.rs)
- Opt-in `AIRA_LLM_SANDBOX_REQUIRED`; `SANDBOX_REQUIRED` / `SANDBOX_REQUIRED_LOOPBACK`
- Operator note in [`docs/local-node.md`](../../docs/local-node.md)
- RFC-0117 stays file-free until `#230`

## 5. Non-Goals

```text
OS vs AIRA-mediated schema rewrite (#229)
RFC-0117 body (#230)
forcing ollama through netns
macOS seatbelt / Windows job objects
changing constraints.network const
HTTP client in ProcessBackend
llama/ggml in aira-core
GPU marketplace
LLM runtime (Core as inference host)
```

## 6. Current Behavior

Landlock/seccomp/netns are independently opt-in. Unset means unsandboxed success. Apply failure of an opted-in layer is already fail-closed.

## 7. Proposed Change

```text
sandbox_required is opt-in, default off.
When set: fail closed if non-Linux, Landlock ABI < 1, or ollama-style loopback.
When set and the host can isolate: apply netns + Landlock + seccomp.
C1 2+2 unchanged (execution-basic).
```

## 8. Affected Books / Schemas / Tests

- Book I §2: host process boundary for LLM Backend CSU
- Schema: unchanged (`aira:schema:execution:generate-local:0.1`) — schema honesty is `#229`
- Tests: `sandbox_required_missing_kernel_is_fail_closed`; `sandbox_required_ollama_is_fail_closed`; `sandbox_required_missing_kernel_is_capsule_failed`; `phase_m_sandbox_required_228`

## 9. Compatibility Impact

Default off. Existing process tests stay unsandboxed unless they opt in. Operators who set the env get fail-closed on hosts that cannot isolate.

## 10. Security Impact

Prevents claiming a completed generate when OS sandbox was required and unavailable. Does not change RFC-0116 adapter `network=none`.

## 11. Privacy Impact

None beyond existing Phase M restricts when the host can apply them.

## 12. Policy Impact

This **is** the sandbox-required policy atom. Activate gate unchanged.

## 13. Failure Semantics

sandbox required + missing kernel / non-Linux MUST fail closed (`SANDBOX_REQUIRED`). sandbox required + ollama-style loopback MUST fail closed (`SANDBOX_REQUIRED_LOOPBACK`). MUST NOT mint VerifiedResultArtifact. MUST NOT succeed unsandboxed when sandbox was required.

## 14. Rollback Plan

Revert this RFC, `sandbox.rs`, and the `with_sandbox_required` hook. Do not create RFC-0117. Do not rewrite generate-local schema in the rollback.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib -- sandbox
cargo test -p aira-desktop-runtime --test phase_m_doc -- phase_m_sandbox_required_228
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Opt-in env; `$id` stays `0.1`.

## 17. Alternatives Considered

- Treat missing kernel as skip / unsandboxed success — rejected (fail-closed).
- File this as RFC-0117 — rejected; RFC-0117 is reserved for `#230`.
- Require sandbox in CI MockBackend — rejected (MockBackend has no child).
- Auto-enable netns on ollama when required — rejected (host loopback exception).

## 18. Evidence

- QUEUE `#228`; [`docs/phase-m-plan.md`](../../docs/phase-m-plan.md) M4
- RFC-0116 remains the adapter `network=none` contract; RFC-0118–0120 remain the opt-in layers

## 19. Open Questions

None for this atom. OS vs AIRA-mediated schema/docs is `#229`.
