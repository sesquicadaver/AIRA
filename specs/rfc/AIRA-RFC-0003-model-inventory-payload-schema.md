# AIRA-RFC-0003 — Local Model Inventory payload schema

## 1. Summary

Additive JSON Schema `aira:schema:model:inventory:0.1` describes a host's local model inventory as `CustomArtifact` content. It does not add `LocalModelInventory` to Core Ontology or a canonical `ArtifactType` enum value.

## 2. Problem Statement

Later inventory CSU (`#58`) must emit an immutable signed snapshot of installed/runnable/incompatible models and cache budget. Without a payload `$id`, scan/list would invent informal JSON.

## 3. Motivation

Phase D (QUEUE `#55`, Analyze-90) needs the third D0 contract after ModelArtifact (`#53`) and ModelProfile (`#54`), so D1 scan/list can target a fixed schema without network or download.

## 4. Scope

- New schema file `schemas/model/inventory.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Envelope remains existing Artifact Descriptor with `artifact_type: CustomArtifact`

## 5. Non-Goals

```text
canonical ArtifactType LocalModelInventoryArtifact
CompatibilityEvidence / AcquisitionPolicy schemas (#56–#57)
aira models scan|list CLI; Inventory CSU runtime (#58)
network fetch; downloader; sharing
Book 0 pipeline change
aira-core / OperationalPlane / C1 change
gpu_id / llm_model_id as Core fields
```

## 6. Current Behavior

`aira:schema:model:artifact:0.1` and `aira:schema:model:profile:0.1` exist. No inventory payload `$id`. No `aira models` CLI.

## 7. Proposed Change

Payload object (not the descriptor) MUST include:

```text
payload_schema = aira:schema:model:inventory:0.1
host_ref (aira:ref)
installed_models, runnable_models, downloadable_compatible_models ([aira:ref])
incompatible_models: [{ model_ref, reason }]
cache_budget: { total_gb, used_gb, reserved_gb } (number ≥ 0)
updated_at (timestamp)
signature
```

`additionalProperties: false`. Missing `signature` MUST fail. Field `downloadable_compatible_models` is inventory state only — this RFC does not authorize download.

## 8. Affected Books / Schemas / Tests

- Schema Pack machine tree: `schemas/model/inventory.schema.json` (additive)
- Fixtures: `fixtures/valid/model/inventory.json`, `fixtures/invalid/model/inventory-missing-signature.json`
- Tests: `aira-schema` fixture manifest + targeted load/validate tests
- Books 0–III: none
- `crates/aira-artifact` `ArtifactType`: none

## 9. Compatibility Impact

Additive. Existing fixtures and C0/C1 unchanged.

## 10. Security Impact

Payload requires `signature` for later immutable inventory snapshots (`#58`). This RFC does not scan filesystem or contact the network.

## 11. Privacy Impact

None beyond host identity ref in fixtures. No user prompts.

## 12. Policy Impact

None. Acquisition policy is `#57` / `#60`.

## 13. Failure Semantics

Invalid payload fails `schema validate --fixtures`. Missing `signature` MUST fail.

## 14. Rollback Plan

Delete the schema file, fixtures, and this RFC; registry walkdir simply stops loading it.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-signature fixture must fail validation.

## 16. Migration Plan

None. New optional payload; producers opt in by setting `CustomArtifact` + this `$id`.

## 17. Alternatives Considered

- New canonical `ArtifactType::LocalModelInventoryArtifact` — rejected until D0 payloads are stable.
- Deferring inventory schema until CLI (`#58`) — rejected; atom rule is schema before CSU.
- Omitting `downloadable_compatible_models` until D4 — rejected; EVO-3 §5.3 lists it as inventory state, not a download action.

## 18. Evidence

- EVO-3 §5.3 Local Model Inventory Artifact field list.
- Book 0 §3.2 / Book I §2: LLM/GPU not Core; may exist as Artifact.
- [`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D0.3.

## 19. Open Questions

Whether to promote `LocalModelInventoryArtifact` into the canonical enum after D0–D3 (separate RFC-S).
