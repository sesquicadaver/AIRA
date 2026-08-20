# AIRA-RFC-0014 — ModelShareOffer payload schema (RFC-S)

## 1. Summary

Additive JSON Schema `aira:schema:model:share-offer:0.1` describes a host-local custom-model share offer as `CustomArtifact` content. It does not add a Core entity or canonical `ArtifactType`. Visibility is `local` | `opt_in` only (no global force).

## 2. Problem Statement

D5 publish/share (`#66`–`#68`) needs a fixed payload `$id` before policy gate and local publish invent informal JSON.

## 3. Scope

- `schemas/model/share-offer.schema.json`
- Valid/invalid fixtures + `fixtures/manifest.json`
- Envelope remains `CustomArtifact`

## 4. Non-Goals

```text
share_custom_models runtime (#66)
local publish CLI (#67)
capability advertisement (#68)
remote registry / DHT / federation push
rating (D6)
canonical ArtifactType::ModelShareOffer
```

## 5. Required fields

```text
payload_schema = aira:schema:model:share-offer:0.1
offer_id, publisher_ref, model_artifact_ref (aira:ref)
content_hash
visibility ∈ {local, opt_in}
allow_download (boolean; producers SHOULD emit false)
created_at, signature
```

Optional: `model_profile_ref`, `license_policy_ref`, `capability_hints`. `additionalProperties: false`. Missing `visibility` MUST fail.

## 6. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6b D5.1; QUEUE `#65`.
