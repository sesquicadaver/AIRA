# AIRA-RFC-0122 — OS isolation vs AIRA-mediated `network=none` (RFC-D)

## 1. Summary

Phase M `#229`: schema and docs **distinguish** two layers:

1. **AIRA-mediated** `constraints.network=none` (RFC-0116) — the generate adapter opens no sockets; encoded in the generate-local payload.
2. **OS child sandbox** (Landlock, seccomp, netns, sandbox-required; RFC-0118–0121) — operator opt-in on `ProcessBackend` / env; **not** encoded in `constraints.network`.

RFC-0117 stays file-free until `#230`. `$id` stays `aira:schema:execution:generate-local:0.1`; `network` const unchanged.

## 2. Problem Statement

After `#225`–`#228`, operators could conflate payload `network=none` with Linux OS isolation. Schema descriptions and operator docs did not explicitly separate adapter contract from optional ProcessBackend layers.

## 3. Motivation

Honest schema descriptions prevent false claims that `network=none` implies Landlock/seccomp/netns. Operators who need OS isolation use explicit env/backend hooks; operators who need adapter-only none keep the RFC-0116 contract.

## 4. Scope

- [`schemas/execution/generate-local.schema.json`](../../schemas/execution/generate-local.schema.json) — `constraints` + `network` descriptions
- Operator note in [`docs/local-node.md`](../../docs/local-node.md)
- Cross-ref in [`docs/csu-development.md`](../../docs/csu-development.md)
- Living test `phase_m_os_vs_aira_mediated_229`
- RFC-0117 stays file-free until `#230`

## 5. Non-Goals

```text
RFC-0117 consolidating body (#230)
changing constraints.network const or $id bump
new ProcessBackend flags or syscall policy
forcing ollama through netns
macOS seatbelt / Windows job objects
HTTP client in ProcessBackend
llama/ggml in aira-core
GPU marketplace
LLM runtime (Core as inference host)
```

## 6. Current Behavior

RFC-0116 adapter `network=none` is enforced in execution-llm. Phase M layers are independently opt-in. Schema already said AIRA-mediated and not OS network-off; `#229` adds explicit Phase M / RFC-0122 separation on `constraints` and cross-docs.

## 7. Proposed Change

```text
constraints.description: AIRA-mediated none vs OS layers not in payload.
network.description: RFC-0116 adapter none; RFC-0122 OS layers are separate operator env.
local-node.md: dedicated OS isolation vs AIRA-mediated subsection (RFC-0122).
No runtime behavior change.
C1 2+2 unchanged (execution-basic).
```

## 8. Affected Books / Schemas / Tests

- Book I §2: host process boundary for LLM Backend CSU
- Schema: description-only update on `aira:schema:execution:generate-local:0.1`
- Tests: `generate_local_payload_schema_loads`; `phase_m_os_vs_aira_mediated_229`; `phase_l_network_none_222` (unchanged needles)

## 9. Compatibility Impact

Description-only schema change. Valid payloads unchanged. `$id` and `network` const unchanged.

## 10. Security Impact

Clarifies threat model: adapter none ≠ OS sandbox. Does not weaken or strengthen either layer.

## 11. Privacy Impact

None.

## 12. Policy Impact

Documentation honesty only. Activate gate unchanged.

## 13. Failure Semantics

Unchanged from RFC-0116 and RFC-0118–0121.

## 14. Rollback Plan

Revert schema descriptions, RFC-0122, and doc cross-refs. Do not create RFC-0117 in rollback.

## 15. Conformance Tests

```text
cargo test -p aira-schema --lib -- generate_local_payload_schema_loads
cargo test -p aira-desktop-runtime --test phase_m_doc -- phase_m_os_vs_aira_mediated_229
cargo test -p aira-desktop-runtime --test phase_l_doc -- phase_l_network_none_222
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Description-only; no payload migration.

## 17. Alternatives Considered

- Bump `$id` to encode OS layers in schema — rejected (operator env, not payload).
- Merge into RFC-0117 at `#229` — rejected (RFC-0117 reserved for `#230`).
- Runtime reject when OS layers unset but `network=none` — rejected (adapter contract only).

## 18. Evidence

- QUEUE `#229`; [`docs/phase-m-plan.md`](../../docs/phase-m-plan.md) M5
- RFC-0116 remains adapter contract; RFC-0118–0121 remain opt-in OS layers

## 19. Open Questions

None for this atom. Consolidating RFC-0117 is `#230`.
