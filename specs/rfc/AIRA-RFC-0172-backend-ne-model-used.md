# AIRA-RFC-0172 — Backend ≠ model used (RFC-D)

## 1. Summary

Phase Q `#283`: Desktop Work `used` model is only model id/hash evidence (`model_ref` / `model_artifact_ref` / `model_content_hash`). Backend names stay in provenance/details — never as `backend:*` in the model field. Without model evidence, used is none/undefined. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`extract_used_model` mapped `result.backend` and mock provenance into the used-model slot (`backend:mock`, `backend:process`, `execution-basic`), conflating LLM backend identity with model identity.

## 3. Motivation

`phase-q-plan` `#283` / acceptance: used model field is not a backend id.

## 4. Scope

- `extract_used_model` evidence-only; reject `backend:`-prefixed refs
- `used_model_fact` defensive filter; `ModelFact::Value` docstring
- QUEUE → `#284`

## 5. Non-Goals

```text
Invite import atomicity (#284)
Requiring model_artifact_ref on every generate (execution CSU emit)
Consolidating RFC-0164
```

## 6. Compatibility / Security

Mock/process envelopes still show honest provenance; System used slot no longer pretends backend is a model.

## 7. Rollout

QUEUE `#283` → Analyze-318 → PR; next `#284` Invite import atomicity.
