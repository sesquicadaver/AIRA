# AIRA-RFC-0002 — Model Profile payload schema

## 1. Summary

Additive JSON Schema `aira:schema:model:profile:0.1` describes model requirement/behaviour payloads as `CustomArtifact` content. It does not add `ModelProfile` to Core Ontology or `ModelProfileArtifact` to the canonical `ArtifactType` enum.

## 2. Problem Statement

Compatibility resolution needs a machine-checkable profile of VRAM/RAM/disk, backends, quantizations, modalities, and domains. Without a payload `$id`, later inventory/resolver CSUs would invent informal JSON.

## 3. Motivation

Phase D (QUEUE `#54`, Analyze-89) needs the second D0 contract after ModelArtifact (`#53`), so D2 compatibility can cite profile fields without treating hardware/LLM as Core.

## 4. Scope

- New schema file `schemas/model/profile.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Envelope remains existing Artifact Descriptor with `artifact_type: CustomArtifact`

## 5. Non-Goals

```text
canonical ArtifactType ModelProfileArtifact
LocalModelInventory / CompatibilityEvidence / AcquisitionPolicy schemas (#55–#57)
hardware scan CLI; inventory CSU; downloader
Book 0 pipeline change
aira-core / OperationalPlane / C1 change
gpu_id / llm_model_id as Core fields
```

## 6. Current Behavior

`aira:schema:model:artifact:0.1` exists. No model profile payload `$id`. No `aira models` CLI.

## 7. Proposed Change

Payload object (not the descriptor) MUST include:

```text
payload_schema = aira:schema:model:profile:0.1
model_ref (aira:ref) → model artifact
required_vram_gb, required_ram_gb, min_disk_gb (number ≥ 0)
supported_backends, supported_quantizations ([string])
context_length (integer ≥ 1)
modalities ∈ {text, vision, audio, code, embedding} (minItems 1)
domains ([string])
estimated_latency_class (string)
evidence_refs ([aira:ref])
```

`additionalProperties: false` (so `gpu_id` / extra Core-like fields fail). Missing `model_ref` MUST fail.

## 8. Affected Books / Schemas / Tests

- Schema Pack machine tree: `schemas/model/profile.schema.json` (additive)
- Fixtures: `fixtures/valid/model/profile.json`, `fixtures/invalid/model/profile-missing-model-ref.json`
- Tests: `aira-schema` fixture manifest + targeted load/validate tests
- Books 0–III: none
- `crates/aira-artifact` `ArtifactType`: none

## 9. Compatibility Impact

Additive. Existing fixtures and C0/C1 unchanged.

## 10. Security Impact

None in this cycle. Profile does not load weights; activation/download remain later rows.

## 11. Privacy Impact

None. No user payloads or prompts.

## 12. Policy Impact

None. Acquisition policy is `#57` / `#60`.

## 13. Failure Semantics

Invalid payload fails `schema validate --fixtures`. Missing `model_ref` MUST fail.

## 14. Rollback Plan

Delete the schema file, fixtures, and this RFC; registry walkdir simply stops loading it.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-`model_ref` fixture must fail validation.

## 16. Migration Plan

None. New optional payload; producers opt in by setting `CustomArtifact` + this `$id`.

## 17. Alternatives Considered

- New canonical `ArtifactType::ModelProfileArtifact` — rejected until D0 payloads are stable (Phase D plan).
- Embedding profile fields into ModelArtifact — rejected; atom rule is one payload schema file per QUEUE row.
- Hardware scan fields as Core Object — forbidden.

## 18. Evidence

- EVO-3 §5.2 Model Profile Artifact field list.
- Book 0 §3.2 / Book I §2: LLM/GPU not Core; may exist as Artifact.
- [`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D0.2.

## 19. Open Questions

Whether to promote `ModelProfileArtifact` into the canonical enum after D0–D3 (separate RFC-S).
