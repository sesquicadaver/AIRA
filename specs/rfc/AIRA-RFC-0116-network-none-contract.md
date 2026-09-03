# AIRA-RFC-0116 — Honest `network=none` generate-local contract (RFC-D)

## 1. Summary

Phase L `#222`: `constraints.network = none` on `text.generate.local` is an **AIRA-mediated** adapter contract. The generate adapter MUST open no sockets. The child process is **not** an OS-isolated network-off sandbox. ollama-style loopback is an explicit host-process exception.

## 2. Problem Statement

RFC-0105 froze `network=none` as `const`. The schema description said “no network,” which operators can read as Landlock / seccomp / network-namespace isolation. ProcessBackend already opens no sockets and documents child loopback (RFC-0110), but the payload schema still sounded like OS enforcement.

## 3. Motivation

Post-K audit: `network=none` is not an OS guarantee. Claiming OS isolation would be a security lie. Honest docs keep operators from treating generate-local as a sandbox.

## 4. Scope

- Schema description on [`constraints.network`](../../schemas/execution/generate-local.schema.json) (same `$id` `0.1`; no const change)
- [`NETWORK_NONE_CONTRACT`](../../csu/execution-llm/src/process.rs) and adapter-source “no sockets” test
- Operator docs: [`docs/local-node.md`](../../docs/local-node.md)
- RFC-0111 stays file-free until `#223`

## 5. Non-Goals

```text
RFC-0111 body (#223)
Landlock / seccomp / Linux network namespace
changing constraints.network const
HTTP client in ProcessBackend
WAN / remote model fetch
llama/ggml in aira-core
GPU marketplace
```

## 6. Current Behavior

Payload validate rejects any `network` other than `"none"`. ProcessBackend uses `Command` only (no `std::net`). Schema description read as blanket “no network.” Child ollama may still use loopback.

## 7. Proposed Change

```text
constraints.network = none means:
  1. AIRA generate adapter opens no sockets (MockBackend in-process; ProcessBackend argv only)
  2. AIRA does not initiate WAN
  3. Child is not OS-isolated; ollama-style loopback is allowed and documented
  4. llama.cpp-style argv remains offline
C1 2+2 unchanged
```

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend CSU; host process boundary
- Schema: description-only on `aira:schema:execution:generate-local:0.1` (no `$id` bump)
- Tests: `process_backend_adapter_does_not_open_sockets`; `network_not_none_is_capsule_failed_aira_mediated`; `generate_local_payload_schema_loads`; `phase_l_network_none_222`

## 9. Compatibility Impact

Payload shape unchanged. Description honesty only. Existing fixtures still validate.

## 10. Security Impact

Removes a false OS-sandbox claim. Does **not** add OS isolation. Operators who need netns/Landlock wait for a post-L atom.

## 11. Privacy Impact

Unchanged: AIRA still does not send generate prompts off-host. A child daemon on loopback is the host’s process, not AIRA WAN.

## 12. Policy Impact

None. Activate gate and acquisition policy unchanged.

## 13. Failure Semantics

`network != "none"` MUST fail closed (`network access forbidden` + [`NETWORK_NONE_CONTRACT`](../../csu/execution-llm/src/process.rs)). MUST NOT mint VerifiedResultArtifact.

## 14. Rollback Plan

Revert schema description and this RFC. Do not create RFC-0111. Do not add Landlock in the rollback.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm --lib -- process_backend_adapter_does_not_open_sockets
cargo test -p aira-csu-execution-llm --lib -- network_not_none_is_capsule_failed_aira_mediated
cargo test -p aira-schema --lib -- generate_local_payload_schema_loads
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
cargo test -p aira-desktop-runtime --test phase_l_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. `$id` stays `0.1`.

## 17. Alternatives Considered

- Implement Landlock/netns now — rejected (Phase L Out; `#223` is RFC-0111 close).
- Drop `network=none` from the payload — rejected (still forbids AIRA sockets / WAN).
- HTTP client to localhost ollama from this crate — rejected (RFC-0110; argv only).
- File this as RFC-0111 — rejected; RFC-0111 is reserved for `#223`.

## 18. Evidence

- QUEUE `#222`; [`docs/phase-l-plan.md`](../../docs/phase-l-plan.md) L5
- RFC-0110 §10 already stated adapter-no-sockets + child loopback; this RFC locks schema/docs to the same reading

## 19. Open Questions

None for this atom.
