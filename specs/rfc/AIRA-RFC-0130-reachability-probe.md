# AIRA-RFC-0130 — Peer-assisted Reachability Probe (RFC-D)

## 1. Summary

Phase N `#238`: signed `ReachabilityChallenge` + external `ReachabilityAttestation` assembled into `ReachabilityResult`. Hairpin/self-connect (`probe_identity == target_identity`) is never proof. Replay log rejects reused challenge ids. Full state machine / `reachability.json` is `#239`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

`bind()` and STUN observation do not prove inbound reachability. Without a peer-assisted signed challenge, Presence cannot honestly claim `DIRECT_REACHABLE`.

## 3. Motivation

Probe is another AIRA node (not a central service). Target signs the challenge; probe signs attestation after exercising the advertised endpoint. Authenticity remains Ed25519 — not EVM payer.

## 4. Scope

- Schema + fixtures: `reachability-challenge.schema.json`
- `aira-peer::reachability` — challenge/attestation/result, hairpin reject, replay, expiry
- Unit tests (roundtrip, hairpin, wrong binding, expired, mutation)
- RFC-D this file; QUEUE advance to `#239`

## 5. Non-Goals

```text
Reachability state machine + reachability.json (#239)
AddressBook promotion (#240)
Two-node external-process harness productization (CI may use later)
Live ledger dial
RFC-0123 consolidating body (#247)
```

## 6. Compatibility / Security

Additive. Discovery ≠ trust. Hairpin forbidden. Replay capped. Does not add ledger/network deps to `aira-core`.

## 7. Rollout

QUEUE `#238` → Analyze-273 → PR; next `#239` Reachability states.
