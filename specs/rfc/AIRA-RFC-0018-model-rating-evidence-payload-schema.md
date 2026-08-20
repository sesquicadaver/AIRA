# AIRA-RFC-0018 — ModelRatingEvidence payload schema (RFC-S / RFC-R)

## 1. Summary

Additive JSON Schema `aira:schema:model:rating-evidence:0.1` describes **context-bound** model rating evidence as `CustomArtifact` content. It does not define a global scalar score or leaderboard entry.

## 2. Problem Statement

D6 needs a fixed payload `$id` before local rating publish / CLI invent informal JSON. EVO-3 requires ratings to be evidence artifacts, not a single global rank.

## 3. Scope

- `schemas/model/rating-evidence.schema.json`
- Valid/invalid fixtures + `fixtures/manifest.json`
- Envelope remains `CustomArtifact`

## 4. Non-Goals

```text
rating CSU publish (#70)
CLI models rate (#71)
upgrade recommendation (D7)
marketplace / popularity / federation score sync
canonical ArtifactType::ModelRatingEvidence
global_score field
```

## 5. Required fields

```text
payload_schema = aira:schema:model:rating-evidence:0.1
model_ref
context { context_id, task_class, … }
reason, confidence, scope, assessed_at
```

Optional: `rater_ref`, dimensional `scores.{fit,latency,quality}`, `evidence_refs`. `additionalProperties: false`. Missing `context` MUST fail. No `global_score` property.

## 6. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6c D6.1; QUEUE `#69`.
