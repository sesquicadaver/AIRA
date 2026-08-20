# AIRA-RFC-0006 — Local Model Inventory CSU (RFC-D)

## 1. Summary

Introduce CSU `aira:csu:model.inventory` (`csu/model-inventory`) that scans a **scoped** local directory and publishes an immutable `CustomArtifact` with payload `$id` `aira:schema:model:inventory:0.1`. Sandbox is `filesystem=scoped`, `network=none` — not a basic CSU.

## 2. Problem Statement

Without an Inventory CSU, CLI cannot produce a signed local inventory snapshot under a declared filesystem scope.

## 3. Motivation

Phase D QUEUE `#58` / Analyze-93 requires Inventory CSU + scoped FS before `aira models scan|list`.

## 4. Scope

- Workspace crate `aira-csu-model-inventory`
- Manifest sandbox: scoped FS under `<root>/models`, network none
- Scan weight files (`.gguf`, `.safetensors`); publish inventory artifact; latest pointer
- CustomEvent `op:inventory-updated:<artifact_id>`

## 5. Non-Goals

```text
C1 plane / OperationalPlane / Book 0 pipeline change
canonical ArtifactType LocalModelInventoryArtifact
compatibility resolver (#59)
download / acquisition (#60 / D4)
network discovery
```

## 6. Proposed Change

`scan_and_publish(root, dir?)` enforces path ⊆ `<root>/models`, builds inventory payload, publishes via `CasArtifactStore`, writes `models/inventory.latest.json`.

## 7. Compatibility Impact

Additive. C1 unchanged. `aira-core` unchanged.

## 8. Security Impact

Scoped FS prevents scanning outside `<root>/models`. Network disabled in sandbox declaration. No download.

## 9. Rollback Plan

Remove crate from workspace and CLI dependency.

## 10. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D1 / RFC-D; EVO-3 §3.1.
