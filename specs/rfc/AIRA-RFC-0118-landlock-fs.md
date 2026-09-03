# AIRA-RFC-0118 — Opt-in Landlock FS restrict for generate-local child (RFC-D)

## 1. Summary

Phase M `#225`: Linux Landlock filesystem restrict on the `ProcessBackend` child. Opt-in only (`ProcessBackend::with_landlock()` or `AIRA_LLM_LANDLOCK=1|true|yes`). Applied after fork, before exec (`pre_exec`). Handled ABI-1 FS accesses are read/execute allowlist; writes, creates, and paths outside the allowlist are denied. Apply failure or non-Linux opt-in is **fail-closed** (`LANDLOCK_FAILED` / `LANDLOCK_UNSUPPORTED`) — never unsandboxed success.

## 2. Problem Statement

Phase L documented that `network=none` is AIRA-mediated, not OS isolation. The generate-local child can still read arbitrary host files (secrets next to the jail). Operators who need an OS FS boundary had no ProcessBackend switch.

## 3. Motivation

A child that can `cat` sibling secrets is not a sandbox. Landlock ABI-1 is the first OS isolation atom (M1). Default stays off so CI MockBackend and existing `/bin/echo` tests remain unsandboxed.

## 4. Scope

- [`csu/execution-llm/src/landlock.rs`](../../csu/execution-llm/src/landlock.rs) + `pre_exec` in [`process.rs`](../../csu/execution-llm/src/process.rs)
- Opt-in `AIRA_LLM_LANDLOCK`; fail-closed constants `LANDLOCK_FAILED` / `LANDLOCK_UNSUPPORTED`
- Operator note in [`docs/local-node.md`](../../docs/local-node.md) (opt-in FS restrict; RFC-0116 adapter `network=none` unchanged)
- RFC-0117 stays file-free until `#230`

## 5. Non-Goals

```text
seccomp syscall filter (#226)
Linux network namespace (#227)
missing-kernel / sandbox-required policy (#228)
OS vs AIRA-mediated schema rewrite (#229)
RFC-0117 body (#230)
changing constraints.network const
HTTP client in ProcessBackend
llama/ggml in aira-core
GPU marketplace
LLM runtime (Core as inference host)
```

## 6. Current Behavior

`ProcessBackend` spawns a fixed argv with env whitelist and bounded pipes. No OS FS restrict. Default and CI remain MockBackend / unsandboxed process.

## 7. Proposed Change

```text
Landlock FS is opt-in, default off.
Linux child: PR_SET_NO_NEW_PRIVS then landlock_restrict_self (ABI-1).
Allowlist: program parent dir + existing runtime dirs
  (/bin, /usr/bin, /lib, /lib64, /usr/lib, /usr/lib64, /etc, /dev).
Grant execute + read file/dir only; handled writes/creates denied.
Opt-in + apply fail or non-Linux → CapsuleFailed, not VERIFIED.
C1 2+2 unchanged (execution-basic).
```

## 8. Affected Books / Schemas / Tests

- Book I §2: host process boundary for LLM Backend CSU
- Schema: unchanged (`aira:schema:execution:generate-local:0.1`)
- Tests: `landlock_denies_read_outside_allowlist`; `landlock_echo_in_jail_succeeds`; `landlock_denied_read_is_capsule_failed`; `phase_m_landlock_225`

## 9. Compatibility Impact

Default off. Existing process tests (`/bin/echo`, dump-env, overflow) stay unsandboxed. Operators enable Landlock explicitly.

## 10. Security Impact

When enabled, the child cannot read files outside the allowlist (fail-closed if restrict cannot be applied). This is **not** seccomp, not netns, and not a claim that RFC-0116 `network=none` is now OS-enforced.

## 11. Privacy Impact

Reduces accidental host-file reads by the child when Landlock is on. Prompts still stay on-host (Phase K/L).

## 12. Policy Impact

None. Activate gate and acquisition policy unchanged. Broader “sandbox required” policy is `#228`.

## 13. Failure Semantics

Landlock opt-in + kernel/path/restrict failure MUST fail closed (`LANDLOCK_FAILED`). Non-Linux opt-in MUST fail closed (`LANDLOCK_UNSUPPORTED`). MUST NOT mint VerifiedResultArtifact. Denied FS read in the child is a non-zero exit → `NONZERO_EXIT` CapsuleFailed.

## 14. Rollback Plan

Revert this RFC, `landlock.rs`, and the `pre_exec` hook. Do not create RFC-0117. Do not add seccomp in the rollback.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib -- landlock
cargo test -p aira-desktop-runtime --test phase_m_doc -- phase_m_landlock_225
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Opt-in env; `$id` stays `0.1`.

## 17. Alternatives Considered

- Always-on Landlock in CI — rejected (MockBackend default; existing echo tests must stay unsandboxed).
- File this as RFC-0117 — rejected; RFC-0117 is reserved for `#230`.
- seccomp or netns in this atom — rejected (linear QUEUE `#226` / `#227`).
- Unsandboxed success when kernel lacks Landlock — rejected (fail-closed).

## 18. Evidence

- QUEUE `#225`; [`docs/phase-m-plan.md`](../../docs/phase-m-plan.md) M1
- RFC-0116 remains the adapter `network=none` contract

## 19. Open Questions

None for this atom. Missing-kernel policy when sandbox is required is `#228`.
