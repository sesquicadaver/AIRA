# AIRA-RFC-0105 — Generate-local payload schema (RFC-S)

## 1. Summary

Additive JSON Schema `aira:schema:execution:generate-local:0.1` describes a host-local `text.generate.local` payload as `CustomArtifact` content. It does not merge into `aira:schema:execution:capsule:0.1`, does not add a Core entity, and does not add a canonical `ArtifactType`.

## 2. Problem Statement

Phase K (`#210`) needs a fixed payload `$id` before `execution-llm` (`#211`) invents informal JSON for generate capsules. Without a contract, later CSU/Reduction/plane atoms cannot name an immutable generate request.

## 3. Motivation

[`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K1 and QUEUE `#210` require schema-first generate before the CSU crate. Book I §2: LLM Backend is a CSU, not Core. Constraints freeze `network=none` and `shell=false` so generate cannot become a command runner or remote fetch.

## 4. Scope

- New schema file `schemas/execution/generate-local.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Envelope remains existing Artifact Descriptor with `artifact_type: CustomArtifact`
- RFC-S only; RFC-0104 stays file-free until `#216`

## 5. Non-Goals

```text
csu/execution-llm crate (#211)
Reduction bind (#212)
OperationalPlane register (#213)
activate gate / required model (#214)
process backend / ollama (#215)
Desktop Work generate path / RFC-0104 (#216)
merging fields into aira:schema:execution:capsule:0.1
llm_model_id / gpu_id as Core fields
aira-core / C1 Calculate 2 + 2 change
```

## 6. Current Behavior

Schema Pack has `aira:schema:execution:capsule:0.1` with unconstrained `constraints`. No generate-local `$id`. No `text.generate.local` payload contract.

## 7. Proposed Change

Payload object (not the capsule descriptor) MUST include:

```text
payload_schema = aira:schema:execution:generate-local:0.1
action const text.generate.local
prompt (string, minLength 1)
constraints.network const "none"
constraints.shell const false
provenance_refs
signature
```

Optional: `problem_statement_ref`, `model_artifact_ref` (aira:ref). Model is not required in 0.1 (`#214` activate gate). `additionalProperties: false` (so `gpu_id` / `llm_model_id` extras fail).

## 8. Affected Books / Schemas / Tests

- Schema Pack machine tree: `schemas/execution/generate-local.schema.json` (additive)
- Fixtures: `fixtures/valid/execution/generate-local.json`, `fixtures/invalid/execution/generate-local-missing-prompt.json`
- Tests: `aira-schema` `fixture_manifest_passes` + `generate_local_payload_schema_loads`; `phase_k_generate_local_210`
- Books 0–III: none
- Canonical `ArtifactType`: none

## 9. Compatibility Impact

Additive. Existing capsule fixtures and C0/C1 unchanged. `Calculate 2 + 2` remains `math.eval.safe`.

## 10. Security Impact

Payload requires `signature` and forbids shell in the data shape. `network=none` is **AIRA-mediated** (adapter opens no sockets); it is **not** OS isolation — see RFC-0116 (`#222`). This RFC does not execute generate, spawn a backend, or load weights.

## 11. Privacy Impact

Valid fixtures include a sample `prompt` string. No user payloads or network export in this RFC.

## 12. Policy Impact

None. Acquisition/activate remain Phase D. Generate without activate is `#214`.

## 13. Failure Semantics

Invalid payload fails `schema validate --fixtures`. Missing `prompt` MUST fail. Extra properties (`gpu_id`) MUST fail.

## 14. Rollback Plan

Delete the schema file, fixtures, this RFC, and the unit tests; registry walkdir simply stops loading it. Do not create RFC-0104.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-prompt fixture must fail validation.

## 16. Migration Plan

None. New optional payload; producers opt in by setting `CustomArtifact` + this `$id`.

## 17. Alternatives Considered

- Merge generate fields into `aira:schema:execution:capsule:0.1` — rejected; capsule stays generic; generate is CustomArtifact content.
- Require `model_artifact_ref` now — rejected; activate gate is `#214`.
- Add `llm_model_id` as a Core field — forbidden (RFC-S MUST NOT add LLM as Core).
- File this as RFC-0104 — rejected; RFC-0104 is reserved for `#216`.

## 18. Evidence

- Book I §2: LLM Backend is a CSU, not Core.
- Book 0 §3.2 / Book IV Non-Goals: reference is not a model hosting system.
- [`docs/phase-k-plan.md`](../../docs/phase-k-plan.md) K1; QUEUE `#210`.

## 19. Open Questions

Whether later atoms add optional sampling fields (`temperature`, `max_tokens`) without relaxing `network`/`shell` — separate RFC-S if needed.
