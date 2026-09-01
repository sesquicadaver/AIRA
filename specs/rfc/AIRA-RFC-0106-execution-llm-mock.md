# AIRA-RFC-0106 — execution-llm CSU + MockBackend (RFC-D)

## 1. Summary

Phase K `#211` adds `csu/execution-llm` (`aira-csu-execution-llm`). A `text.generate.local` capsule (RFC-0105) completes with `CapsuleCompleted` only when a [`MockBackend`](../../csu/execution-llm/src/lib.rs) (or later process backend) is bound. No backend, invalid payload, wrong action, or extra properties → `CapsuleFailed`. This is not a fake VERIFIED result.

## 2. Problem Statement

Schema `#210` names generate-local JSON, but nothing executes it. Without a CSU, later Reduction/plane atoms have no fail-closed generate path, and a missing backend could be papered over as success.

## 3. Motivation

[`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K2 and QUEUE `#211`: LLM Backend is a CSU, not Core. Tests need a deterministic mock that never shells out or uses the network. Default construction must fail closed.

## 4. Scope

- Workspace member `csu/execution-llm`
- `ExecutionLlmCsu` + `GenerateBackend` + `MockBackend`
- Named crate tests for complete / missing-backend / invalid payload
- RFC-0104 stays file-free until `#216`

## 5. Non-Goals

```text
Reduction bind (#212)
OperationalPlane / LocalSession register (#213)
activate gate / required model (#214) — TODO fail-closed hook only
process backend / ollama / llama.cpp (#215)
Desktop Work generate path / RFC-0104 (#216)
Cargo dep on model-inventory or model-acquisition
llama/ggml in aira-core
C1 Calculate 2 + 2 / math.eval.safe change
GPU marketplace
```

## 6. Current Behavior

Only `execution-basic` handles `CapsuleCreated` (`math.eval.safe` / `text.echo` / `text.uppercase`). No generate CSU.

## 7. Proposed Change

```text
CapsuleCreated + generate-local payload + MockBackend → CapsuleCompleted + ExecutionArtifact
no backend bound → CapsuleFailed (not VERIFIED)
invalid / extra properties / wrong action → CapsuleFailed
sandbox network=none; mock never Command/network
```

Optional `model_artifact_ref` is accepted per RFC-0105 and not gated here (`#214`).

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend = CSU
- Schema: none (RFC-0105 unchanged)
- Tests: `cargo test -p aira-csu-execution-llm`; `phase_k_execution_llm_211`
- Canonical `ArtifactType`: none (input CustomArtifact; output ExecutionArtifact)

## 9. Compatibility Impact

Additive crate. C1 2+2 and `execution-basic` unchanged. Plane does not dispatch to this CSU until `#213`.

## 10. Security Impact

Fail-closed without backend. Strict `deny_unknown_fields` parse. No shell, no network, no Core inference.

## 11. Privacy Impact

Mock echoes the prompt into a deterministic `mock-generate:…` string. No user traffic leaves the host.

## 12. Policy Impact

None. Activate remains `#214`. Acquisition remains Phase D.

## 13. Failure Semantics

Missing backend / invalid payload MUST emit `CapsuleFailed` + `CsuOutput::Failure`. MUST NOT emit `VerifiedResultArtifact`.

## 14. Rollback Plan

Remove `csu/execution-llm` from the workspace, this RFC, Analyze-246, and living-spec `#211` rows. Do not create RFC-0104.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm
cargo test -p aira-desktop-runtime --test phase_k_doc
```

C0/C1 `Calculate 2 + 2` is not this crate.

## 16. Migration Plan

None. Crate is unused by the plane until `#213`.

## 17. Alternatives Considered

- Default-bind MockBackend in `new()` — rejected; fail-closed without backend is the atom contract.
- Put mock in `aira-core` — forbidden (anti-mission).
- Depend on inventory CSU for model presence — CSU↛CSU; activate is `#214`.
- File this as RFC-0104 — rejected; RFC-0104 is reserved for `#216`.

## 18. Evidence

- Book I §2: LLM Backend is a CSU, not Core.
- [`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K2; QUEUE `#211`.
- RFC-0105 generate-local payload.

## 19. Open Questions

Process argv adapter shape (`#215`) — out of this RFC.
