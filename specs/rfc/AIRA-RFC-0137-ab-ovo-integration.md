# AIRA-RFC-0137 — Ab ovo publish→discover→trust→dial (RFC-D)

## 1. Summary

Phase N `#245`: integration path where Node A publishes Presence to a shared rendezvous ledger and Node B — with **empty** AddressBook — discovers A, records DISCOVERED, admits trust explicitly, promotes AddressBook, and establishes a mutual Noise session. NAT/relay blocked-inbound is `#246`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Phase N atoms were unit-scoped. TZ §47 requires an end-to-end acceptance test without preconfigured peers in B's AddressBook.

## 3. Motivation

Prove zero-knowledge discovery + trust gate + dial without PeerInvite fixtures.

## 4. Scope

- `aira-peer::ab_ovo` — `record_discovered_presence`, `admit_peer_trust`, `discover_admit_promote`
- `DiscoverySource::Rendezvous`
- Integration test: empty ledger → A publish → B discover → trust → dial Noise
- RFC-D this file; QUEUE → `#246`

## 5. Non-Goals

```text
NAT/relay dual-blocked Noise (#246)
RFC-0123 consolidating body (#247)
Auto-trust from ledger
Live EVM JSON-RPC dial
```

## 6. Compatibility / Security

Additive. `DISCOVERED ≠ TRUSTED`. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#245` → Analyze-280 → PR; next `#246` NAT/relay integration.
