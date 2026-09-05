# AIRA-RFC-0131 — Reachability states + reachability.json (RFC-D)

## 1. Summary

Phase N `#239`: persist local reachability status (`UNKNOWN`…`OFFLINE`) in `peers/reachability.json`. `DIRECT_REACHABLE` only after a verified peer-assisted probe (`#238`). Local bind alone stays `LOCAL_ONLY` and must not advertise DIRECT. AddressBook promotion is `#240`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Probe results need durable local status so Presence refresh and operators know whether inbound is proven. Without a state file, every restart returns to UNKNOWN without evidence linkage.

## 3. Motivation

TZ §17 states + §33 `reachability.json` fields: status, checked_at, local_port, observed/verified endpoints, relay_routes, probe_evidence.

## 4. Scope

- `aira-peer::reachability_state` — `ReachabilityStatus`, `ReachabilityLocalState`, transitions
- Persist `peers/reachability.json`
- Unit tests (local bind ≠ DIRECT, probe → DIRECT, relay/outbound/offline)
- RFC-D this file; QUEUE advance to `#240`

## 5. Non-Goals

```text
AddressBook promotion (#240)
CLI reachability commands (#243)
Relay product integration (#241)
RFC-0123 consolidating body (#247)
```

## 6. Compatibility / Security

Additive. Does not upsert TrustStore. Does not add ledger deps to `aira-core`.

## 7. Rollout

QUEUE `#239` → Analyze-274 → PR; next `#240` AddressBook promotion.
