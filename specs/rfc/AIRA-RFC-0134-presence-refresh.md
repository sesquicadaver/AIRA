# AIRA-RFC-0134 — Presence refresh / endpoint change (RFC-D)

## 1. Summary

Phase N `#242`: Presence TTL refresh (`sequence++`, new `created_at`/`expires_at`), stale filter by `expires_at`, and endpoint-change republication that advertises only new endpoints. AddressBook peer ids listed for notify; live peer-protocol dial remains `#243`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Without refresh, Presence expires and stale endpoints accumulate as if still current. Port/NAT/relay changes must bump sequence and drop old advertisements.

## 3. Motivation

TZ §24–§25: regular republication; endpoint change verifies new path, increments sequence, publishes, stops advertising old endpoints, notifies trusted peers via existing peer protocol.

## 4. Scope

- `aira-peer::presence_refresh` — `refresh_and_sign_presence`, `endpoint_change_and_sign_presence`, `retain_unexpired_presence`, `trusted_peers_to_notify`
- Default TTL 1d within rendezvous min/max (`PRESENCE_REFRESH_TTL_SECS_DEFAULT`)
- Unit tests; RFC-D this file; QUEUE → `#243`

## 5. Non-Goals

```text
CLI peer commands (#243)
Desktop UX (#244)
Live dial notify of peers (orchestration in #243+)
RFC-0123 consolidating body (#247)
```

## 6. Compatibility / Security

Additive. Does not upsert TrustStore. Does not add ledger deps to `aira-core`.

## 7. Rollout

QUEUE `#242` → Analyze-277 → PR; next `#243` CLI.
