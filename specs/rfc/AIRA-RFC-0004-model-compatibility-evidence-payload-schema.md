# AIRA-RFC-0004 — Model Compatibility Evidence payload schema

## 1. Summary

Additive JSON Schema `aira:schema:model:compatibility-evidence:0.1` describes a host-local model compatibility classification as `CustomArtifact` content. It does not add a Core entity or a new canonical `ArtifactType` enum value.

## 2. Problem Statement

D2 compatibility resolver (`#59`) must emit evidence with `reason`, `confidence`, and `scope` for each model (`runnable` / `incompatible` / `unknown`). Without a payload `$id`, later CSUs would invent informal JSON or misuse rating scores.

## 3. Motivation

Phase D (QUEUE `#56`, Analyze-91) needs the fourth D0 contract so D2 can cite a fixed schema without auto-download or global rating.

## 4. Scope

- New schema file `schemas/model/compatibility-evidence.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Envelope remains existing Artifact Descriptor with `artifact_type: CustomArtifact`

## 5. Non-Goals

```text
canonical ArtifactType ModelCompatibilityEvidence
AcquisitionPolicy schema (#57)
compatibility resolver runtime (#59)
auto-download; rating score / ModelRatingEvidence
Book 0 pipeline change
aira-core / OperationalPlane / C1 change
gpu_id / llm_model_id as Core fields
```

## 6. Current Behavior

Model artifact / profile / inventory payload schemas exist. No compatibility-evidence payload `$id`. No `aira models compatible` CLI.

## 7. Proposed Change

Payload object (not the descriptor) MUST include:

```text
payload_schema = aira:schema:model:compatibility-evidence:0.1
model_ref (aira:ref)
compatibility ∈ {runnable, incompatible, unknown}
reason (string, minLength 1)
confidence (number 0..1)
scope (aira:schema:common:scope-descriptor:0.1)
```

Optional: `profile_ref`, `host_ref`, `assessed_at`, `evidence_refs`. `additionalProperties: false`. Missing `reason` MUST fail. No rating-score fields.

## 8. Affected Books / Schemas / Tests

- Schema Pack machine tree: `schemas/model/compatibility-evidence.schema.json` (additive)
- Fixtures: `fixtures/valid/model/compatibility-evidence.json`, `fixtures/invalid/model/compatibility-evidence-missing-reason.json`
- Tests: `aira-schema` fixture manifest + targeted load/validate tests
- Books 0–III: none
- `crates/aira-artifact` `ArtifactType`: none

## 9. Compatibility Impact

Additive. Existing fixtures and C0/C1 unchanged.

## 10. Security Impact

None in this cycle. Payload does not load weights or authorize download.

## 11. Privacy Impact

None beyond optional host ref in fixtures. No user prompts.

## 12. Policy Impact

None. Acquisition policy is `#57` / `#60`.

## 13. Failure Semantics

Invalid payload fails `schema validate --fixtures`. Missing `reason` MUST fail.

## 14. Rollback Plan

Delete the schema file, fixtures, and this RFC; registry walkdir simply stops loading it.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-reason fixture must fail validation.

## 16. Migration Plan

None. New optional payload; producers opt in by setting `CustomArtifact` + this `$id`.

## 17. Alternatives Considered

- Reuse canonical `EvidenceArtifact` only — rejected; QUEUE atom requires a model-layer payload with explicit reason/confidence/scope and compatibility enum for D2.
- Embedding a scalar `rating_score` — forbidden by QUEUE Out and Phase D plan (rating is D6 / RFC-R).
- Implementing the resolver in this row — rejected; atom rule is schema before `#59`.

## 18. Evidence

- [`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D0.4 / D2.
- EVO-3: runnable / downloadable-compatible / incompatible classification; inventory reasons.
- Epistemic pattern: confidence + scope (Book / Schema Pack).

## 19. Open Questions

Whether resolver `#59` wraps this payload inside a signed `EvidenceArtifact` envelope or emits `CustomArtifact` only (separate Analyze).
