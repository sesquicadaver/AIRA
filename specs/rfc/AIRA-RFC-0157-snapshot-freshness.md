# AIRA-RFC-0157 — Snapshot freshness (RFC-D)

## 1. Summary

Phase P `#267`: Desktop `SystemSnapshot` distinguishes **measurement time** (`reachability.checked_at` → `observed_at`) from **projection load time** (`loaded_at`). Network `DataQuality` can be `Stale` / `Unknown` / `Unavailable` / `Current`. Configured or recorded bind is never treated as proven live listener. RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

Phase O `#256` always marked network quality `Current` when identity existed, and set `observed_at` to load clock (`unix:<secs>`). Status strip mapped DIRECT/RELAYED to «connected» regardless of observation age. `local_bind` mixed settings listen with reachability `local_port` without provenance.

## 3. Motivation

`phase-p-plan` P1 and `desktop-ux` require name-of-state = verified fact; Stale/Unavailable must not paint as Current connection.

## 4. Scope

- `NetworkMeshSnapshot`: `reachability_checked_at`, `LocalBindProvenance`, `local_listener_proven` (Desktop always `false` until a live accept-loop feed)
- `SystemSnapshot::{observed_at, loaded_at}` + `classify_network_quality` (stale after `NETWORK_OBSERVATION_STALE_SECS`)
- Strip / System UI: Stale → «stale»; Unknown/Unavailable → «not checked»; show loaded vs observed
- Unit tests + living smoke advance to first OPEN `#268`

## 5. Non-Goals

```text
Applied-from-runtime (#268)
Model triple (#269)
Reachability endpoint+direction attestation (#270)
Consolidating RFC-0156 (#274)
```

## 6. Compatibility / Security

No Core/ledger changes. Rendezvous «connected» remains local publish metadata only (UI labels it as such).

## 7. Rollout

QUEUE `#267` → Analyze-302 → PR; next `#268` Applied from runtime.
