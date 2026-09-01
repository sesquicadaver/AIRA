# AIRA-RFC-0109 — Generate-local Phase D activate gate (RFC-D)

## 1. Summary

Phase K `#214`: `execution-llm` fail-closes `text.generate.local` unless a [`ModelActivateGate`](../../csu/execution-llm/src/lib.rs) reports that a model is Phase D activated. Outcome is `CapsuleFailed` (+ FailureEvidence on the plane), never a fake VERIFIED result. The CSU does not Cargo-depend on inventory/acquisition; the plane or tests inject the handle (same pattern as `GenerateBackend`).

## 2. Problem Statement

After `#213`, generate-local completes via MockBackend with no Phase D activate check. A host without an activated model could still look like a successful generate.

## 3. Motivation

[`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K5 and QUEUE `#214`: no activate / inactive model → CapsuleFailed + Evidence, not VERIFIED. Activation remains the Phase D D4 `#64` pointer (`models/activated.latest.json`). RFC-0105 `model_artifact_ref` stays optional on the payload; the gate is **activation state**.

## 4. Scope

- `ModelActivateGate` + `AlwaysActivated` / `NeverActivated` test doubles on `execution-llm`
- Default: no bound gate → fail-closed
- Plane injects the handle: tests use `AlwaysActivated`; `LocalSession` uses `ActivatedPointerGate`
- Named tests for both generate paths
- RFC-0104 stays file-free until `#216`

## 5. Non-Goals

```text
process backend / ollama / llama.cpp (#215)
Desktop Work generate path / RFC-0104 (#216)
Cargo dep execution-llm → inventory/acquisition (CSU ↛ CSU)
real model download / inventory mutation
changing Reduction catalog
llama/ggml in aira-core
GPU marketplace
requiring model_artifact_ref on every generate-local fixture
```

## 6. Current Behavior

`activate_gate_placeholder` always returned `Ok`. MockBackend + plane register completed generate without Phase D activate.

## 7. Proposed Change

```text
execution-llm:
  no ModelActivateGate bound → CapsuleFailed (ACTIVATE_DENIED)
  NeverActivated → CapsuleFailed
  AlwaysActivated + MockBackend → CapsuleCompleted (unchanged #213 success)
  CSU ↛ CSU: trait injected by plane/tests

OperationalPlane:
  default open() fail-closed (no gate)
  enable_activated_mock_llm() for tests/CI mock path
  LocalSession binds ActivatedPointerGate on models/activated.latest.json

C1 Calculate 2 + 2:
  execution-llm skips math.eval.safe; activate gate not consulted
```

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend = CSU; activation is Phase D, not Core
- Schema: none (RFC-0105 `model_artifact_ref` remains optional)
- Tests: `inactive_model_is_capsule_failed`; `never_activated_gate_is_capsule_failed`; `mock_backend_completes_valid_generate_local`; `generate_without_activate_is_capsule_failed`; `non_math_prompt_completes_via_execution_llm_mock`; `phase_d_activated_pointer_allows_mock_generate`; `calculate_two_plus_two_stays_execution_basic`; `phase_k_activate_gate_214`
- Canonical `ArtifactType`: none

## 9. Compatibility Impact

C1 2+2 unchanged. Generate without activate now fails on the default plane (intended). Tests that expect mock generate must inject `AlwaysActivated` or a Phase D pointer.

## 10. Security Impact

Fail-closed without activate. No shell, no network, no Core inference, no inventory mutation.

## 11. Privacy Impact

None beyond existing MockBackend prompt echo.

## 12. Policy Impact

Generate now requires Phase D activate state. Acquisition/download policy unchanged.

## 13. Failure Semantics

No gate / inactive pointer / mismatched `model_artifact_ref` MUST emit `CapsuleFailed`. On the plane, evidence-basic MUST emit `FailureEvidenceCreated`. MUST NOT emit `VerifiedResultArtifact`.

## 14. Rollback Plan

Revert the trait/gate wiring, this RFC, Analyze-249, and living-spec `#214` rows. Do not create RFC-0104.

## 15. Conformance Tests

```text
cargo test -p aira-csu-execution-llm
cargo test -p aira-flow --lib
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_k_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / `execution-basic`.

## 16. Migration Plan

None. Additive fail-closed gate. Producers that already activated a model via Phase D keep working when the pointer is present.

## 17. Alternatives Considered

- Cargo-dep execution-llm on model-acquisition — rejected (CSU ↛ CSU).
- Require `model_artifact_ref` on every RFC-0105 payload — rejected; gate is activation state.
- Default-bind `AlwaysActivated` on the reference plane — rejected; that would skip the fail-closed contract.
- File this as RFC-0104 — rejected; RFC-0104 is reserved for `#216`.

## 18. Evidence

- Book I §2: LLM Backend is a CSU, not Core.
- Phase D `#64` / `models/activated.latest.json`.
- [`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K5; QUEUE `#214`.
- RFC-0105 generate-local payload; RFC-0106 execution-llm mock; RFC-0108 plane register.

## 19. Open Questions

Process argv adapter (`#215`) — out of this RFC.
