# AIRA-RFC-0127 — RendezvousProvider abstraction (RFC-D)

## 1. Summary

Phase N `#235`: define ledger-agnostic `RendezvousProvider` in `aira-peer` with a deterministic in-memory `MockRendezvousProvider` for CI. No EVM types, no `aira-core` ledger deps. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Presence Records need a publish/query substrate, but Core and peer code must not embed chain-specific logic. Without a trait + mock, later EVM adapter and CI harness cannot share one API.

## 3. Motivation

Abstraction keeps distributed ledger as external rendezvous memory only. Mock satisfies Phase N invariant: CI uses mock + harness; Amoy is not a merge gate.

## 4. Scope

- `aira-peer::rendezvous` — `RendezvousProvider` trait + `MockRendezvousProvider`
- Operations: `publish_presence`, `update_presence`, `remove_or_expire_presence`, `query_active_peers`, `query_identity`, `query_relays`, `provider_kind`
- Mock admits only canonically signed Presence; sequence must increase on update; no TrustStore upsert
- RFC-D this file; QUEUE advance to `#236`

## 5. Non-Goals

```text
EvmRendezvousProvider / Amoy/mainnet hooks (#236)
Full ledger TTL/sequence product rules beyond mock (#237)
Reachability / AddressBook promotion (#238–#240)
RFC-0123 consolidating body (#247)
aira-core ledger or network deps
```

## 6. API sketch

```text
trait RendezvousProvider {
  publish_presence(record)
  update_presence(record)
  remove_or_expire_presence(identity_ref, as_of, force)
  query_active_peers(as_of)
  query_identity(identity_ref)
  query_relays(as_of)
  provider_kind() -> "mock" | …
}
```

## 7. Compatibility / Security

Additive. Discovery ≠ trust (`DISCOVERED ≠ TRUSTED`). Signature verify uses Presence embedded pubkey only.

## 8. Rollout

QUEUE `#235` → Analyze-270 → PR; next `#236` EVM ledger adapter.
