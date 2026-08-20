# AIRA-RFC-0008 — Model compatibility resolver (RFC-D/E)

## 1. Summary

CSU `aira:csu:model.compatibility` classifies installed models as `runnable` / `incompatible` / `unknown` using a local host profile and optional model profiles. Each classification is published as a `CustomArtifact` with payload `aira:schema:model:compatibility-evidence:0.1`. CLI: `aira models compatible`.

## 2. Problem Statement

D2 needs evidence-backed reasons without download or Core hardware entities.

## 3. Scope

- Crate `csu/model-compatibility` (sandbox: `filesystem=read_only`, `network=none`)
- Inputs: inventory pointer from `#58`; optional `models/host.profile.json`; optional `models/profiles/*.json`
- Outputs: one evidence artifact per installed model + `models/compatibility.latest.json`
- CLI `aira models compatible`

## 4. Non-Goals

```text
download / acquisition CSU (#60 / D4)
C1 plane change
canonical ArtifactType
rating score
auto-download
```

## 5. Classification rules

```text
missing host or model profile → unknown + reason
required_vram/ram/disk exceeds host → incompatible + reason
no overlapping backends → incompatible + reason
otherwise → runnable + reason
```

## 6. Compatibility Impact

Additive. Does not depend on `aira-csu-model-inventory` (CSU↛CSU). Reads shared on-disk inventory pointer.

## 7. Rollback

Remove crate + CLI `Compatible` variant.

## 8. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D2; QUEUE `#59`.
