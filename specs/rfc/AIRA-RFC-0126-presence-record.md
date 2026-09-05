# AIRA-RFC-0126 — Node Presence Record (RFC-D)

## 1. Summary

Phase N `#234`: introduce signed `NodePresenceRecord` (`aira:schema:peer:presence-record:0.1`) with canonical Ed25519 over the descriptor without `signature`. Mutation of port/host/identity/expiry/sequence/relay breaks verify. Ledger publish is `#235+`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Global rendezvous needs a stable, authentic advertisement of identity + endpoints before AddressBook trust. Without a Presence type, later RendezvousProvider atoms have no payload contract.

## 3. Motivation

Canonical signature binds `identity_ref` to endpoints and sequence. Verify uses embedded `identity_public_key` (no TrustStore upsert — `DISCOVERED ≠ TRUSTED`).

## 4. Scope

- Schema + fixtures (`schemas/peer/presence-record.schema.json`)
- `aira-peer::presence` — draft/sign/verify/validate_shape (P_AIRA ports, `aira:network:public:v1`)
- Mutation unit tests
- RFC-D this file; QUEUE advance to `#235`

## 5. Non-Goals

```text
RendezvousProvider trait (#235)
EVM ledger publish/query (#236–#237)
Reachability state machine (#238–#239)
AddressBook promotion (#240)
sequence monotonic ledger checks beyond shape (sequence >= 1)
RFC-0123 consolidating body (#247)
```

## 6. Wire fields

```text
schema, network_id, identity_ref, identity_public_key, sequence,
created_at, expires_at, direct_endpoints[], relay_endpoints[],
capabilities_hash, signature
```

Direct: `transport` ∈ {tcp-peer, udp-discv}, host, port ∈ P_AIRA, reachability_state, observed_at.  
Relay: relay_identity_ref, relay_endpoint (P_AIRA), reservation_id, expires_at.

## 7. Compatibility / Security

Additive. Presence verify does not imply trust. Public key in record is the verify material.

## 8. Rollout

QUEUE `#234` → Analyze-269 → PR; next `#235` RendezvousProvider.
