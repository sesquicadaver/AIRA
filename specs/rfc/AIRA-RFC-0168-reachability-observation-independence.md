# AIRA-RFC-0168 — Reachability observation independence (RFC-D)

## 1. Summary

Phase Q `#279`: local bind, external DIRECT, and relay keep independent observation times. `mark_local_bind` must not refresh external/relay freshness. Desktop `classify_network_quality` treats future clocks beyond a skew bound as `Unknown`, not `Current`. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

A single `checked_at` was updated by local bind even when status stayed DIRECT/RELAY, so Desktop showed external reachability as Current after a local-only bind. Any future `checked_at` was also treated as Current with no skew cap.

## 3. Motivation

`phase-q-plan` `#279` / acceptance: local bind must not refresh external DIRECT as Current; future clock ≠ Current without skew bound.

## 4. Scope

- `local_checked_at` / `external_checked_at` / `relay_checked_at` on `ReachabilityLocalState`
- `status_observation_at()` for banner freshness
- `mark_local_bind` preserves external/relay clocks when status is DIRECT/RELAY
- `NETWORK_OBSERVATION_MAX_SKEW_SECS` in mesh quality classification
- QUEUE → `#280`

## 5. Non-Goals

```text
Reachability NAT endpoints (#280)
Evidence admission (#281)
Schema bump of REACHABILITY_STATE_SCHEMA (additive fields with serde default)
```

## 6. Compatibility / Security

Pre-`#279` files with only `checked_at` still load; `status_observation_at` falls back to legacy field.

## 7. Rollout

QUEUE `#279` → Analyze-314 → PR; next `#280` Reachability NAT endpoints.
