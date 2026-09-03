# AIRA-RFC-0114 — ProcessBackend bounded pipes (RFC-D)

## 1. Summary

Phase L `#220`: `ProcessBackend` must cap stdout and stderr **while reading**. Overflow MUST be `CapsuleFailed`, never a truncated CapsuleCompleted / fake VERIFIED.

## 2. Problem Statement

`#215` used `read_to_end` on both pipes. A child that streams unbounded output can grow host memory without bound. Truncating after the fact still allocated the whole buffer.

## 3. Motivation

Post-K audit: unbounded pipes are an execution-boundary gap. Env whitelist (`#219`) does not cap IO.

## 4. Scope

- [`read_bounded`](../../csu/execution-llm/src/process.rs) during generate
- stdout [`PIPE_STDOUT_LIMIT`](../../csu/execution-llm/src/process.rs) (1 MiB); stderr [`PIPE_STDERR_LIMIT`](../../csu/execution-llm/src/process.rs) (64 KiB)
- Overflow kills the child and returns [`PIPE_OVERFLOW`](../../csu/execution-llm/src/process.rs)
- RFC-0111 stays file-free until `#223`

## 5. Non-Goals

```text
ProblemRecord execution_artifact_id (#221)
network=none RFC (#222)
RFC-0111 body (#223)
Landlock / seccomp
configurable limits via env
llama/ggml in aira-core
GPU marketplace
```

## 6. Current Behavior

Both pipes are `read_to_end` with no cap. stderr is truncated only when formatting a non-zero-exit error string.

## 7. Proposed Change

```text
ProcessBackend::generate:
  read stdout/stderr in chunks
  if len would exceed limit → PIPE_OVERFLOW (not truncated success)
  kill child on overflow so a blocked writer unblocks
AlwaysActivated / MockBackend unchanged
C1 2+2 unchanged
```

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend CSU; host process boundary
- Schema: none
- Tests: `read_bounded_overflow_during_read`; `stdout_overflow_during_read_is_fail_closed`; `stderr_overflow_during_read_is_fail_closed`; `stdout_overflow_is_capsule_failed`; `phase_l_bounded_pipes_220`

## 9. Compatibility Impact

Children that emit more than 1 MiB stdout (or 64 KiB stderr) fail closed. Operators needing larger logs wait for a later atom.

## 10. Security Impact

Bounds host memory against a runaway local CLI. This is not OS isolation.

## 11. Privacy Impact

None beyond existing generate-local stdout → ExecutionArtifact.

## 12. Policy Impact

None.

## 13. Failure Semantics

Overflow MUST emit `CapsuleFailed`. MUST NOT mint VerifiedResultArtifact. MUST NOT return truncated generate text as success.

## 14. Rollback Plan

Revert bounded read to `read_to_end`. Do not create RFC-0111.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_l_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Default mock path does not spawn.

## 17. Alternatives Considered

- `read_to_end` then truncate — rejected (audit: cap **during** read).
- Treat overflow as truncated success — rejected (looks like VERIFIED/Completed with incomplete text).
- Configurable env limits — out of this atom.

## 18. Evidence

- QUEUE `#220`; [`docs/phase-l-plan.md`](../../docs/phase-l-plan.md) L3
- RFC-0110 unbounded `read_to_end` (superseded for pipe depth)

## 19. Open Questions

None for this atom.
