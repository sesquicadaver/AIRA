# AIRA-RFC-0159 — Model triple UX (selected ≠ ready ≠ used) (RFC-D)

## 1. Summary

Phase P `#269`: Desktop surfaces three independent model facts — **selected** (Phase D `activated.latest.json` `model_ref`), **ready** (full activate evidence/hash confirmation), **used-in-result** (from the last Work payload only). Strip, System, and Settings observe the triple; Settings is observe-only (not an inventory editor). RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

Strip always showed «not checked»; System ignored `view.model`; Settings treated a placeholder as the Models control. Selected could be confused with used.

## 3. Motivation

`phase-p-plan` and post-O audit: selected ≠ prepared ≠ used in the result.

## 4. Scope

- `ActivatedPointerGate::observe` / `ActivationObservation`
- `ModelTripleSnapshot` + strip/System/Settings UI
- `WorkResultView.used_model` extraction
- Tests; QUEUE → `#270`

## 5. Non-Goals

```text
Reachability endpoint bind (#270)
Full inventory editor / GPU marketplace
Consolidating RFC-0156 (#274)
```

## 6. Compatibility / Security

No Core/ledger changes. Ready never means VERIFIED. C1 `Calculate 2 + 2` still needs no model.

## 7. Rollout

QUEUE `#269` → Analyze-304 → PR; next `#270` Reachability endpoint bind.
