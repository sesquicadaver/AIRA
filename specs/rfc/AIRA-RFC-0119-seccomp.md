# AIRA-RFC-0119 — Opt-in seccomp deny-list for generate-local child (RFC-D)

## 1. Summary

Phase M `#226`: Linux seccomp BPF filter on the `ProcessBackend` child. Opt-in only (`ProcessBackend::with_seccomp()` or `AIRA_LLM_SECCOMP=1|true|yes`). Applied after fork, before exec (`pre_exec`), after Landlock when both are enabled. Default-allow with a deny-list (socket/ptrace/mount/module/unshare/…). Denied syscall → `SECCOMP_RET_KILL_PROCESS` (SIGSYS) → CapsuleFailed (`SECCOMP_VIOLATION`). Apply failure or non-Linux opt-in is **fail-closed** (`SECCOMP_FAILED` / `SECCOMP_UNSUPPORTED`).

## 2. Problem Statement

Landlock (`#225`) restricts filesystem paths. The child can still issue network, ptrace, mount, and namespace syscalls. Operators who need a syscall boundary had no ProcessBackend switch.

## 3. Motivation

A child that can `socket()` is not an OS network-off sandbox, and RFC-0116 must stay honest. seccomp M2 is the syscall atom. Default stays off so CI MockBackend and existing `/bin/echo` tests remain unfiltered.

## 4. Scope

- [`csu/execution-llm/src/seccomp.rs`](../../csu/execution-llm/src/seccomp.rs) + `pre_exec` in [`process.rs`](../../csu/execution-llm/src/process.rs)
- Opt-in `AIRA_LLM_SECCOMP`; `SECCOMP_FAILED` / `SECCOMP_UNSUPPORTED` / `SECCOMP_VIOLATION`
- Operator note in [`docs/local-node.md`](../../docs/local-node.md)
- RFC-0117 stays file-free until `#230`

## 5. Non-Goals

```text
Linux network namespace (#227)
missing-kernel / sandbox-required policy (#228)
OS vs AIRA-mediated schema rewrite (#229)
RFC-0117 body (#230)
seccomp allowlist of every glibc syscall
changing constraints.network const
HTTP client in ProcessBackend
llama/ggml in aira-core
GPU marketplace
LLM runtime (Core as inference host)
```

## 6. Current Behavior

Opt-in Landlock FS may restrict paths. No syscall filter. Default and CI remain MockBackend / unsandboxed process.

## 7. Proposed Change

```text
seccomp is opt-in, default off.
Linux child: PR_SET_NO_NEW_PRIVS then SECCOMP_SET_MODE_FILTER.
Default-allow BPF; deny socket/ptrace/mount/module/unshare/setns/…
Forbidden syscall → SIGSYS → CapsuleFailed (SECCOMP_VIOLATION).
Opt-in + apply fail or non-Linux → CapsuleFailed, not VERIFIED.
C1 2+2 unchanged (execution-basic).
```

## 8. Affected Books / Schemas / Tests

- Book I §2: host process boundary for LLM Backend CSU
- Schema: unchanged (`aira:schema:execution:generate-local:0.1`)
- Tests: `seccomp_forbidden_syscall_is_fail_closed`; `seccomp_echo_succeeds`; `seccomp_forbidden_syscall_is_capsule_failed`; `phase_m_seccomp_226`

## 9. Compatibility Impact

Default off. Existing process tests stay unfiltered. Operators enable seccomp explicitly. ollama loopback `socket()` is blocked when seccomp is on (netns/loopback exception remains `#227` / RFC-0116 honesty).

## 10. Security Impact

When enabled, a denied syscall kills the child. This is **not** netns, not a full allowlist sandbox, and not a claim that RFC-0116 `network=none` is OS-enforced.

## 11. Privacy Impact

Reduces accidental network syscalls by the child when seccomp is on. Prompts still stay on-host (Phase K/L).

## 12. Policy Impact

None. Activate gate unchanged. Broader “sandbox required” policy is `#228`.

## 13. Failure Semantics

seccomp opt-in + filter install failure MUST fail closed (`SECCOMP_FAILED`). Non-Linux opt-in MUST fail closed (`SECCOMP_UNSUPPORTED`). SIGSYS MUST fail closed (`SECCOMP_VIOLATION`). MUST NOT mint VerifiedResultArtifact.

## 14. Rollback Plan

Revert this RFC, `seccomp.rs`, and the `pre_exec` hook. Do not create RFC-0117. Do not add netns in the rollback.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib -- seccomp
cargo test -p aira-desktop-runtime --test phase_m_doc -- phase_m_seccomp_226
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Opt-in env; `$id` stays `0.1`.

## 17. Alternatives Considered

- Always-on seccomp in CI — rejected (MockBackend default; echo tests must stay unfiltered).
- File this as RFC-0117 — rejected; RFC-0117 is reserved for `#230`.
- netns in this atom — rejected (linear QUEUE `#227`).
- Full syscall allowlist — rejected (too brittle for `/bin/sh` + echo in this atom; deny-list is the measurable M2).
- Unsandboxed success when seccomp cannot be installed — rejected (fail-closed).

## 18. Evidence

- QUEUE `#226`; [`docs/phase-m-plan.md`](../../docs/phase-m-plan.md) M2
- RFC-0116 remains the adapter `network=none` contract; RFC-0118 remains Landlock

## 19. Open Questions

None for this atom. netns for offline argv is `#227`.
