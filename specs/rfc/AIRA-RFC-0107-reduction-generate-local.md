# AIRA-RFC-0107 — Reduction generate-local catalog bind (RFC-D)

## 1. Summary

Phase K `#212`: `reduction-basic` binds non-math / non-echo / non-upper Problem Statements to `text.generate.local` with RFC-0105 payload `aira:schema:execution:generate-local:0.1`. `Calculate 2 + 2` stays `math.eval.safe`. Bind is by **action/capability string**; this crate does not Cargo-depend on `execution-llm` (CSU ↛ CSU). Plane dispatch is `#213`.

## 2. Problem Statement

After `#211`, generate-local capsules exist but Reduction still defaulted every non-echo/non-upper statement to `math.eval.safe` (and often rewrote it as `2+2`). Non-math prompts could not select the execution-llm action.

## 3. Motivation

[`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K3 and QUEUE `#212`: catalog split before the plane registers execution-llm. C1 `c1.pipeline.calculate_2_plus_2` must remain `execution-basic`.

## 4. Scope

- `catalog_action` in `csu/reduction-basic`
- Generate-local CustomArtifact payload (RFC-0105 fields only; `network=none`, `shell=false`)
- Named tests: `calculate_2_plus_2_binds_math_eval_safe`; `non_math_prompt_binds_generate_local`
- RFC-0104 stays file-free until `#216`

## 5. Non-Goals

```text
OperationalPlane / LocalSession register (#213)
activate gate / required model (#214)
process backend / ollama / llama.cpp (#215)
Desktop Work generate path / RFC-0104 (#216)
Cargo dep reduction-basic → execution-llm
changing execution-basic math path
llama/ggml in aira-core
```

## 6. Current Behavior

```text
echo → text.echo
upper → text.uppercase
else → math.eval.safe  (including prose prompts; missing operators defaulted expression to 2+2)
```

## 7. Proposed Change

```text
echo → text.echo
upper → text.uppercase
Calculate … with a digit, or bare arithmetic → math.eval.safe (ExecutionArtifact capsule)
else → text.generate.local (CustomArtifact, RFC-0105 payload)
required_capabilities / CapsuleCreated.payload_ref = action string
constraints.network = none; constraints.shell = false
```

## 8. Affected Books / Schemas / Tests

- Book III RED-001 catalog bind (action selection only)
- Schema: none (RFC-0105 unchanged)
- Tests: `cargo test -p aira-csu-reduction-basic`; `phase_k_reduction_bind_212`; C1 `c1.pipeline.calculate_2_plus_2`
- Canonical `ArtifactType`: generate uses existing `CustomArtifact`

## 9. Compatibility Impact

C1 2+2 and echo/uppercase unchanged. Non-math prompts on the current plane still hit `execution-basic` until `#213` (unsupported action → CapsuleFailed). That is expected; this atom only selects the action.

## 10. Security Impact

Generate payload freezes `network=none` / `shell=false`. No CSU→CSU import. No Core inference.

## 11. Privacy Impact

Prompt text is the Problem Statement already on the event. No network export.

## 12. Policy Impact

None. Activate remains `#214`. Acquisition remains Phase D.

## 13. Failure Semantics

Reduction still emits CapsuleCreated. Execution of generate is `#213`. Invalid extra properties are not produced (`additionalProperties: false` payload).

## 14. Rollback Plan

Revert `catalog_action` to the prior else→`math.eval.safe` default, this RFC, Analyze-247, and living-spec `#212` rows. Do not create RFC-0104.

## 15. Conformance Tests

```text
cargo test -p aira-csu-reduction-basic
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_k_doc
cargo test -p aira-flow --lib calculate_two_plus_two
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

None. Catalog is local Reduction behavior. Plane still dispatches CapsuleCreated to execution-basic until `#213`.

## 17. Alternatives Considered

- Default remaining prompts to math.eval.safe — rejected; that is the bug this atom removes.
- Cargo-depend on `aira-csu-execution-llm` for ACTION const — rejected; CSU ↛ CSU; bind by string like other catalog entries.
- File this as RFC-0104 — rejected; RFC-0104 is reserved for `#216`.

## 18. Evidence

- Book I §2: LLM Backend is a CSU, not Core.
- [`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K3; QUEUE `#212`.
- RFC-0105 generate-local payload; RFC-0100 durable reuse catalog (unchanged).

## 19. Open Questions

None for this atom. Plane register (`#213`) consumes the action selected here.
