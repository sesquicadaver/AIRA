# AIRA-RFC-0021 — UpgradeRecommendation payload schema (RFC-S / RFC-R)

## 1. Summary

Additive JSON Schema `aira:schema:model:upgrade-recommendation:0.1` describes an **advisory** upgrade recommendation as `CustomArtifact` content. Evidence-backed; not marketplace/settlement.

## 2. Required

`recommendation_id`, `recommendation_type` ∈ {hardware,model,storage,backend,none}, `reason`, `evidence_refs` (min 1), `confidence`, `scope`, `created_at`.

## 3. Non-Goals

CSU publish (`#73`), CLI (`#74`), marketplace, auto-download, ads.

## 4. Evidence

phase-d-plan §6d D7.1; QUEUE `#72`.
