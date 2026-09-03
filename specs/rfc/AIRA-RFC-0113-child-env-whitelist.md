# AIRA-RFC-0113 — ProcessBackend child env whitelist (RFC-D)

## 1. Summary

Phase L `#219`: `ProcessBackend` must `env_clear` the child and copy only PATH, HOME, and LANG. `AIRA_HTTP_TOKEN` (and other host secrets) MUST NOT appear in the child environment.

## 2. Problem Statement

`#215` spawned the CLI with the inherited process environment. A Desktop/node host that has Bearer auth in `AIRA_HTTP_TOKEN` would leak that secret to ollama/llama.cpp (or any generate-local argv).

## 3. Motivation

Post-K audit: child env inheritance is an execution-boundary gap. Activate evidence (`#218`) does not cover process isolation of secrets.

## 4. Scope

- [`ProcessBackend`](../../csu/execution-llm/src/process.rs): `Command::env_clear` + PATH/HOME/LANG
- Tests: allowlist keys; spawned child does not inherit `AIRA_HTTP_TOKEN`
- RFC-0111 stays file-free until `#223`

## 5. Non-Goals

```text
bounded stdout/stderr during read (#220)
ProblemRecord execution_artifact_id (#221)
network=none RFC (#222)
RFC-0111 body (#223)
Landlock / seccomp / network namespace
llama/ggml in aira-core
GPU marketplace
```

## 6. Current Behavior

Child inherits the full parent environment, including `AIRA_HTTP_TOKEN` when set.

## 7. Proposed Change

```text
ProcessBackend::generate:
  Command::env_clear()
  copy PATH (parent, else /usr/bin:/bin)
  copy HOME if set
  copy LANG (parent, else C)
  do not copy AIRA_HTTP_TOKEN / AIRA_LLM_* / other host env
AlwaysActivated / MockBackend unchanged
C1 2+2 unchanged
```

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend CSU; host process boundary
- Schema: none
- Tests: `child_env_pairs_never_include_http_token`; `process_child_does_not_inherit_http_token`; `phase_l_child_env_219`; existing `echo_process_backend_completes_without_ollama`

## 9. Compatibility Impact

Children that required arbitrary inherited env (proxy vars, `USER`, `TERM`) may need those in a later atom. This atom is fail-closed on secrets, not a general sandbox.

## 10. Security Impact

Bearer token and other host env are not passed to the generate CLI. This is not OS isolation.

## 11. Privacy Impact

Fewer host identifiers reach the child. Prompt still goes as argv.

## 12. Policy Impact

None. Acquisition policy unchanged.

## 13. Failure Semantics

Unchanged from RFC-0110: spawn/nonzero/timeout/empty stdout → `CapsuleFailed`. Env whitelist does not mint VERIFIED.

## 14. Rollback Plan

Revert `env_clear` / allowlist in `process.rs`. Do not create RFC-0111.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_l_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Operators who relied on leaking host env into the LLM CLI must stop doing so.

## 17. Alternatives Considered

- Copy all non-`AIRA_*` keys — rejected (too wide; audit asked for whitelist).
- Drop PATH too — rejected; nested tools (ollama helpers) need PATH.
- Landlock — out (`#222` / post-L).

## 18. Evidence

- QUEUE `#219`; [`docs/phase-l-plan.md`](../../docs/phase-l-plan.md) L2
- RFC-0110 process spawn (env inheritance superseded)

## 19. Open Questions

None for this atom.
