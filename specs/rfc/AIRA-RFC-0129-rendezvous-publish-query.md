# AIRA-RFC-0129 — Rendezvous publish/query (TTL/sequence) (RFC-D)

## 1. Summary

Phase N `#237`: product publish/query layer (`RendezvousClient`) over `RendezvousProvider` with TTL bounds, record size limit, query caps, EVM publish call encoding, and local `peers/rendezvous.json` metadata. Live JSON-RPC dial remains deferred (local double / mock). RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Trait + EVM adapter alone do not enforce anti-spam TTL/sequence/size policy or persist local rendezvous service metadata. Callers need a single admission path before ledger storage.

## 3. Motivation

Anti-spam from TZ §29: record TTL, monotonic sequence, size limit, query limit, signature before persistence. Local `rendezvous.json` holds service metadata only (not a ledger mirror).

## 4. Scope

- `aira-peer::rendezvous_ops` — `RendezvousClient`, `RendezvousPublishPolicy`, `RendezvousLocalState`, `EvmPublishCall`
- Defaults: TTL 60s–7d; max record 64KiB; max query 256
- RFC-D this file; QUEUE advance to `#238`

## 5. Non-Goals

```text
Live Polygon JSON-RPC dial (still local double)
Reachability probe (#238)
AddressBook promotion (#240)
CLI (`#243`)
RFC-0123 consolidating body (#247)
```

## 6. Compatibility / Security

Additive. Signature verify before persist. Discovery ≠ trust. EVM payer ≠ AIRA identity. Does not add ledger deps to `aira-core`.

## 7. Rollout

QUEUE `#237` → Analyze-272 → PR; next `#238` Reachability Probe.
