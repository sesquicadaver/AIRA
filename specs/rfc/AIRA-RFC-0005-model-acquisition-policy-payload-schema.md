# AIRA-RFC-0005 — Model Acquisition Policy payload schema

## 1. Summary

Additive JSON Schema `aira:schema:model:acquisition-policy:0.1` describes a host-local model acquisition policy as `CustomArtifact` content. It does not add a Core entity or a new canonical `ArtifactType` enum value. Default posture is `auto_download=false`.

## 2. Problem Statement

D3 policy runtime (`#60`) must DENY download when no policy exists, and evaluate an explicit policy artifact when present. Without a payload `$id`, later CSUs would invent informal JSON or bake download into Core.

## 3. Motivation

Phase D (QUEUE `#57`, Analyze-92) needs the fifth D0 contract so D3 can cite a fixed schema with default-deny auto-download, without implementing the downloader or allowlist runtime.

## 4. Scope

- New schema file `schemas/model/acquisition-policy.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Envelope remains existing Artifact Descriptor with `artifact_type: CustomArtifact`

## 5. Non-Goals

```text
canonical ArtifactType ModelAcquisitionPolicyArtifact
downloader / activation flow (D4)
allowlist runtime evaluation
Inventory CLI (#58); compatibility resolver (#59); policy DENY runtime (#60)
Book 0 pipeline change
aira-core / OperationalPlane / C1 change
gpu_id / llm_model_id as Core fields
```

## 6. Current Behavior

Model artifact / profile / inventory / compatibility-evidence payloads exist. No acquisition-policy payload `$id`. No `aira models policy` CLI.

## 7. Proposed Change

Payload object (not the descriptor) MUST include:

```text
payload_schema = aira:schema:model:acquisition-policy:0.1
host_ref (aira:ref)
auto_download (boolean; default false — producers SHOULD emit false)
allow_untrusted_models (boolean; default false)
share_custom_models (boolean; default false — explicit opt-in)
updated_at (timestamp)
signature
```

Optional: `allow_download_if_size_below_gb`, `allow_quantized_only`, `max_model_cache_size_gb`. `additionalProperties: false`. Missing `auto_download` MUST fail.

## 8. Affected Books / Schemas / Tests

- Schema Pack machine tree: `schemas/model/acquisition-policy.schema.json` (additive)
- Fixtures: `fixtures/valid/model/acquisition-policy.json`, `fixtures/invalid/model/acquisition-policy-missing-auto-download.json`
- Tests: `aira-schema` fixture manifest + targeted load/validate tests
- Books 0–III: none
- `crates/aira-artifact` `ArtifactType`: none

## 9. Compatibility Impact

Additive. Existing fixtures and C0/C1 unchanged.

## 10. Security Impact

Schema encodes default-deny posture (`auto_download=false`). This RFC does not perform download, network I/O, or policy evaluation.

## 11. Privacy Impact

None beyond host identity ref in fixtures. No user prompts.

## 12. Policy Impact

Defines the Model Acquisition Policy payload contract. Runtime DENY-without-policy is `#60`. Sharing remains opt-in (`share_custom_models=false` by default).

## 13. Failure Semantics

Invalid payload fails `schema validate --fixtures`. Missing `auto_download` MUST fail.

## 14. Rollback Plan

Delete the schema file, fixtures, and this RFC; registry walkdir simply stops loading it.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-`auto_download` fixture must fail validation. Valid fixture MUST have `auto_download: false`.

## 16. Migration Plan

None. New optional payload; producers opt in by setting `CustomArtifact` + this `$id`.

## 17. Alternatives Considered

- Const `auto_download: false` only — rejected; users may later set true via explicit policy `#60`/`policy set`, but schema must still require the field.
- Embedding allowlist runtime arrays — deferred; QUEUE Out forbids allowlist runtime in this row.
- Implementing downloader here — forbidden (D4 / not in D0–D3 first wave).

## 18. Evidence

- EVO-3 §2 example: `allow_auto_download = false by default`.
- [`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) invariant 4 and D0.5.

## 19. Open Questions

Exact mapping from this payload to `#60` PolicyDenied Event fields (separate Analyze).
