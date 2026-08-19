# AIRA-RFC-0001 — Model Artifact payload schema

## 1. Summary

Additive JSON Schema `aira:schema:model:artifact:0.1` describes local model-weight payloads as `CustomArtifact` content. It does not add `Model` to Core Ontology or `ModelArtifact` to the canonical `ArtifactType` enum.

## 2. Problem Statement

Nodes will hold different model files. Without a payload contract, inventory/acquisition cannot name an immutable hashed model object except by informal JSON.

## 3. Motivation

Phase D (QUEUE `#53`, Analyze-88) needs a machine-checkable payload so later inventory CSU can point at model files without treating LLM/GPU as Core.

## 4. Scope

- New schema file `schemas/model/artifact.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Envelope remains existing Artifact Descriptor with `artifact_type: CustomArtifact`

## 5. Non-Goals

```text
canonical ArtifactType ModelArtifact
Model Profile / Inventory / Policy schemas (#54–#57)
downloader, sharing, rating, recommendation
Book 0 pipeline change
aira-core / OperationalPlane / C1 change
llm_model_id / gpu_id as Core fields
```

## 6. Current Behavior

Schema Pack and `ArtifactType` have `CustomArtifact` but no model payload `$id`. No `aira models` CLI.

## 7. Proposed Change

Payload object (not the descriptor) MUST include:

```text
payload_schema = aira:schema:model:artifact:0.1
model_id (aira:ref)
format ∈ {gguf, safetensors, custom}
quantization, parameter_class
content_hash (sha256|sha512)
provenance_refs
signature
```

Optional: `tokenizer_ref`, `license_policy_ref`. `additionalProperties: false` (so `gpu_id` / extra Core-like fields fail).

## 8. Affected Books / Schemas / Tests

- Schema Pack machine tree: `schemas/model/artifact.schema.json` (additive)
- Fixtures: `fixtures/valid/model/artifact.json`, `fixtures/invalid/model/artifact-missing-hash.json`
- Tests: `aira-schema` `fixture_manifest_passes` + targeted load/validate tests
- Books 0–III: none
- `crates/aira-artifact` `ArtifactType`: none

## 9. Compatibility Impact

Additive. Existing fixtures and C0/C1 unchanged.

## 10. Security Impact

Payload requires `content_hash` and `signature` fields for later verify-before-activation (D4). This RFC does not activate or load weights.

## 11. Privacy Impact

None. No user payloads or prompts.

## 12. Policy Impact

None. Acquisition policy is `#57` / `#60`.

## 13. Failure Semantics

Invalid payload fails `schema validate --fixtures`. Missing `content_hash` MUST fail.

## 14. Rollback Plan

Delete the schema file, fixtures, and this RFC; registry walkdir simply stops loading it.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-hash fixture must fail validation.

## 16. Migration Plan

None. New optional payload; producers opt in by setting `CustomArtifact` + this `$id`.

## 17. Alternatives Considered

- New canonical `ArtifactType::ModelArtifact` — rejected until the payload is stable (Phase D plan).
- Informal JSON without schema — rejected; CI cannot gate it.
- Putting model fields on Core Object Descriptor — forbidden (RFC-S MUST NOT add `llm_model_id` as core dependency).

## 18. Evidence

- Book 0 §3.2 / Book I §2: LLM/GPU not Core; may exist as Artifact.
- Book IV Non-Goals: reference is not a model hosting system.
- [`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D0.1.

## 19. Open Questions

Whether to promote `ModelArtifact` into the canonical enum after D0–D3 (separate RFC-S).
