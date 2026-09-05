# AIRA-RFC-0133 — Relay integration dial path + advertisements (RFC-D)

## 1. Summary

Phase N `#241`: product dial order **direct → NAT observed → relay**, signed prime-port `RelayAdvertisement`, and dual reservation SHOULD (`RELAY_RESERVATION_TARGET=2`) for RELAY_ONLY. Relay remains transport fallback, not discovery authority. Live two-node NAT/relay Noise smoke is `#246`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Presence promotion and reachability states need a concrete dial planner and trusted relay ads so operators can fall back without inventing ad-hoc paths or auto-trusting ledger relays.

## 3. Motivation

TZ §19–§20: direct fail → NAT attempt → relay selection; RELAY_ONLY SHOULD keep ≥2 independent reservations; ads are signed and trust-gated; courier never verifies endpoint payloads.

## 4. Scope

- `aira-peer::relay_integrate` — `plan_dial_path`, `RelayAdvertisement`, `RelayAdStore` (`peers/relay_ads.json`), `select_relay_reservations`
- Prime-port fail-closed on all AIRA endpoints in the path
- Unit tests (order, dual trust, untrusted skip, expire, non-prime reject)
- RFC-D this file; QUEUE advance to `#242`

## 5. Non-Goals

```text
Presence refresh (#242)
CLI (#243)
Ab ovo / NAT Noise integration (#245–#246)
RFC-0123 consolidating body (#247)
Auto TrustStore upsert from relay ads
```

## 6. Compatibility / Security

Additive. Does not add ledger deps to `aira-core`. Does not change existing `RelayHub` courier semantics. `DISCOVERED ≠ TRUSTED`.

## 7. Rollout

QUEUE `#241` → Analyze-276 → PR; next `#242` Presence refresh.
