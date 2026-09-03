# AIRA-RFC-0120 — Opt-in Linux network namespace for generate-local child (RFC-D)

## 1. Summary

Phase M `#227`: Linux network namespace on the `ProcessBackend` child. Opt-in only (`ProcessBackend::with_netns()` or `AIRA_LLM_NETNS=1|true|yes`). Applied after fork, before exec (`pre_exec`), **before** Landlock and seccomp (needs `/proc/self/{uid,gid}_map` and `unshare`; the `#226` filter denies `SYS_unshare` / `SYS_setns`). Unprivileged path: `CLONE_NEWUSER` + uid/gid map, then `CLONE_NEWNET`. Apply failure or non-Linux opt-in is **fail-closed** (`NETNS_FAILED` / `NETNS_UNSUPPORTED`). Combining netns with ollama-style host loopback is **fail-closed** (`NETNS_BLOCKS_LOOPBACK`) so host `127.0.0.1` is not silently isolated.

## 2. Problem Statement

Landlock (`#225`) and seccomp (`#226`) restrict FS and syscalls. Offline argv can still use the host network namespace. Operators who need OS net isolation for llama.cpp-style children had no ProcessBackend switch. Applying netns to ollama-style children would break the host loopback daemon without an explicit error.

## 3. Motivation

A child that shares the host netns is not an OS network-off sandbox, and RFC-0116 must stay honest. netns M3 is the network-namespace atom for **offline argv**. ollama loopback stays an explicit exception: netns + ollama fail-closes instead of isolating silently. Default stays off so CI MockBackend and existing `/bin/echo` tests remain un-namespaced unless opted in.

## 4. Scope

- [`csu/execution-llm/src/netns.rs`](../../csu/execution-llm/src/netns.rs) + `pre_exec` order in [`process.rs`](../../csu/execution-llm/src/process.rs)
- Opt-in `AIRA_LLM_NETNS`; `NETNS_FAILED` / `NETNS_UNSUPPORTED` / `NETNS_BLOCKS_LOOPBACK`
- Operator note in [`docs/local-node.md`](../../docs/local-node.md)
- RFC-0117 stays file-free until `#230`

## 5. Non-Goals

```text
missing-kernel / sandbox-required policy (#228)
OS vs AIRA-mediated schema rewrite (#229)
RFC-0117 body (#230)
forcing ollama through netns
bringing up child lo / veth to host
changing constraints.network const
HTTP client in ProcessBackend
llama/ggml in aira-core
GPU marketplace
LLM runtime (Core as inference host)
```

## 6. Current Behavior

Opt-in Landlock FS and opt-in seccomp may restrict the child. No network namespace. Default and CI remain MockBackend / host netns process. ollama-style children may use host loopback when netns is off.

## 7. Proposed Change

```text
netns is opt-in, default off.
Linux child pre_exec order: netns → Landlock → seccomp.
unshare(CLONE_NEWUSER) → setgroups deny + uid/gid map → unshare(CLONE_NEWNET).
ollama() / basename ollama + netns → CapsuleFailed (NETNS_BLOCKS_LOOPBACK) before spawn.
Opt-in + apply fail or non-Linux → CapsuleFailed, not VERIFIED.
C1 2+2 unchanged (execution-basic).
```

## 8. Affected Books / Schemas / Tests

- Book I §2: host process boundary for LLM Backend CSU
- Schema: unchanged (`aira:schema:execution:generate-local:0.1`)
- Tests: `ollama_with_netns_is_fail_closed`; `netns_isolates_host_loopback`; `netns_echo_succeeds_or_fail_closed`; `ollama_netns_is_capsule_failed`; `phase_m_netns_227`

## 9. Compatibility Impact

Default off. Existing process tests stay on the host netns. Operators enable netns explicitly for offline argv. ollama + netns is an explicit fail, not a silent loopback break.

## 10. Security Impact

When enabled on offline argv, the child does not share the host network namespace. This is **not** RFC-0116 OS `network=none`, not a claim that Landlock/seccomp are implied, and not a path that may silently strand ollama.

## 11. Privacy Impact

Reduces accidental WAN/LAN use by offline argv when netns is on. Prompts still stay on-host (Phase K/L).

## 12. Policy Impact

None. Activate gate unchanged. Broader “sandbox required” policy is `#228`.

## 13. Failure Semantics

netns opt-in + unshare/map failure MUST fail closed (`NETNS_FAILED`). Non-Linux opt-in MUST fail closed (`NETNS_UNSUPPORTED`). ollama-style + netns MUST fail closed (`NETNS_BLOCKS_LOOPBACK`). MUST NOT mint VerifiedResultArtifact. MUST NOT succeed unsandboxed when netns was requested.

## 14. Rollback Plan

Revert this RFC, `netns.rs`, and the `pre_exec` hook. Do not create RFC-0117. Do not add missing-kernel policy in the rollback.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib -- netns
cargo test -p aira-desktop-runtime --test phase_m_doc -- phase_m_netns_227
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Opt-in env; `$id` stays `0.1`.

## 17. Alternatives Considered

- Always-on netns in CI — rejected (MockBackend default; ollama exception must stay explicit).
- File this as RFC-0117 — rejected; RFC-0117 is reserved for `#230`.
- Silent netns on ollama — rejected (host loopback would break without CapsuleFailed).
- veth/lo setup so ollama still reaches host — rejected (out of this atom; exception is fail-closed).
- Unsandboxed success when unshare cannot be applied — rejected (fail-closed).

## 18. Evidence

- QUEUE `#227`; [`docs/phase-m-plan.md`](../../docs/phase-m-plan.md) M3
- RFC-0116 remains the adapter `network=none` contract; RFC-0118 Landlock; RFC-0119 seccomp

## 19. Open Questions

None for this atom. missing-kernel / sandbox-required policy is `#228`.
