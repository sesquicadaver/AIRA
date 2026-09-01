# AIRA-RFC-0108 — Plane register execution-llm (RFC-D)

## 1. Summary

Phase K `#213`: `OperationalPlane` / `LocalSession` register `aira-csu-execution-llm` with [`MockBackend`](../../csu/execution-llm/src/lib.rs). A non-math Problem Statement that Reduction bound to `text.generate.local` (RFC-0107) completes with `CapsuleCompleted` and a human-readable mock result. This is **not** a Verified Result Artifact. C1 `Calculate 2 + 2` stays `execution-basic` / `math.eval.safe`.

## 2. Problem Statement

After `#212`, generate-local capsules are selected but the plane only dispatched `CapsuleCreated` to `execution-basic` (unsupported action → `CapsuleFailed`). Generate never ran.

## 3. Motivation

[`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K4 and QUEUE `#213`: the CSU exists; the reference plane must invoke it the same way other execution CSUs are wired (workspace dep, factory, activate). Tests/CI use MockBackend. Process/ollama is `#215`.

## 4. Scope

- Register `ExecutionLlmCsu::new().with_mock_backend()` on `OperationalPlane`
- Fan-out skip: execution-basic ignores `text.generate.local`; execution-llm ignores math/echo/uppercase capsules (generate-local schema + wrong action still fail-closed)
- `SubmitOutcome::Executed` for generate-local (no fake `VERIFIED`)
- Named tests: `non_math_prompt_completes_via_execution_llm_mock`; `calculate_two_plus_two_stays_execution_basic`
- RFC-0104 stays file-free until `#216`

## 5. Non-Goals

```text
activate gate / required Phase D model (#214)
process backend / ollama / llama.cpp (#215)
Desktop Work generate path / RFC-0104 (#216)
Cargo dep execution-llm → inventory/acquisition (CSU ↛ CSU)
changing Reduction catalog logic
llama/ggml in aira-core
GPU marketplace
```

## 6. Current Behavior

Reduction emits generate-local `CapsuleCreated`. Plane handlers are context / reduction / execution-basic / verification / evidence / epistemic. Generate fails at execution-basic.

## 7. Proposed Change

```text
OperationalPlane handlers += ExecutionLlmCsu + MockBackend
CapsuleCreated fan-out:
  math.eval.safe / text.echo / text.uppercase → execution-basic
  text.generate.local → execution-llm MockBackend → CapsuleCompleted + ExecutionArtifact
  verification-basic skips generate-local (no fake VERIFIED)
submit_problem:
  VRA present → Completed (C1 2+2 unchanged)
  else generate-local ExecutionArtifact → Executed
```

`#214` activate-gate placeholder on execution-llm remains a TODO hook.

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend = CSU on the operational plane
- Schema: none
- Tests: `cargo test -p aira-flow --lib`; `cargo test -p aira-conformance --lib`; `phase_k_plane_register_213`
- Canonical `ArtifactType`: none (ExecutionArtifact output already used by execution-llm)

## 9. Compatibility Impact

C1 2+2 unchanged. Echo/uppercase still VERIFIED via execution-basic. Prose prompts return `Executed` instead of `CapsuleFailed`. CLI/HTTP report `status executed` without `verification_status: VERIFIED`.

## 10. Security Impact

MockBackend never shells out or uses the network. Generate constraints stay `network=none` / `shell=false`. No Core inference.

## 11. Privacy Impact

Mock echoes the prompt into `mock-generate:…`. No user traffic leaves the host.

## 12. Policy Impact

None. Activate remains `#214`. Acquisition remains Phase D.

## 13. Failure Semantics

Invalid generate-local payload still `CapsuleFailed`. Missing backend still fail-closed (plane binds MockBackend). MUST NOT emit `VerifiedResultArtifact` for generate-local.

## 14. Rollback Plan

Unregister execution-llm from the plane, revert `SubmitOutcome::Executed`, this RFC, Analyze-248, and living-spec `#213` rows. Do not create RFC-0104.

## 15. Conformance Tests

```text
cargo test -p aira-flow --lib
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_k_doc
cargo test -p aira-csu-execution-llm
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / `execution-basic`.

## 16. Migration Plan

None. Additive handler. Existing Completed path unchanged.

## 17. Alternatives Considered

- Route CapsuleCreated by action in CsuRuntime — rejected; existing pattern is fan-out + CSU skip.
- Return `Completed` with a fake VRA for mock text — rejected; atom forbids fake VERIFIED.
- File this as RFC-0104 — rejected; RFC-0104 is reserved for `#216`.

## 18. Evidence

- Book I §2: LLM Backend is a CSU, not Core.
- [`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K4; QUEUE `#213`.
- RFC-0105 generate-local payload; RFC-0106 execution-llm mock; RFC-0107 Reduction bind.

## 19. Open Questions

Activate-without-inventory (`#214`) — out of this RFC. Process argv adapter (`#215`) — out of this RFC.
