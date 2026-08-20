# AIRA-RFC-0016 — local publish signed descriptor (RFC-D/E)

## 1. Summary

After `share_custom_models` ALLOW, `publish_local` materializes a signed ModelArtifact descriptor and a linked ShareOffer from the activated cache (`models/cache` via `models/activated.latest.json`), publishes both as `CustomArtifact` to CAS, writes `models/share-offer.latest.json`, and emits `op:share-published:…`. No remote push / network.

## 2. Problem Statement

D5.3 requires a host-local publish path that turns an activated weight into signed share descriptors without capability advertisement or remote registry.

## 3. Scope

- `publish_local` in `csu/model-acquisition`
- Pointer `models/share-offer.latest.json`
- CLI: `aira models publish --model-ref [--visibility local|opt_in] [--allow-download]`
- Event `op:share-published:publish:{model_ref}:{visibility}`

## 4. Non-Goals

```text
capability advertisement (#68)
remote registry / HTTP share
rating (D6)
global visibility
```

## 5. Flow

1. `request_publish` gate (DENY → exit; no ShareOffer).
2. Require activated pointer matching `model_ref` + cache file under scoped `models/`.
3. Sign ModelArtifact (`artifact:0.1`) from cache hash; CAS id `aira:artifact:model-desc:…`.
4. Sign ShareOffer (`share-offer:0.1`) linking `model_artifact_ref`; CAS id `aira:artifact:share-offer:…`.
5. Pointer + Event. `network=none`.

## 6. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6b D5.3; QUEUE `#67`.
