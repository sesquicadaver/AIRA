# AIRA-RFC-0125 — Deterministic preferred port selection (RFC-D)

## 1. Summary

Phase N `#233`: select AIRA-owned listen ports as
`P_AIRA[H(identity_ref || transport_class || version) mod 1491]`.
On bind collision, walk `index+1…` with wrap over `P_AIRA` (finite; full wrap → error).
Presence / rendezvous remain `#234+`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

`#232` validates membership in `P_AIRA` but does not assign a stable preferred port per identity. Operators and two-node discovery need a deterministic, allocator-free preference.

## 3. Motivation

Same identity → same preferred port across restarts; no central port allocator; collisions resolve by walking the next primes.

## 4. Scope

- `crates/aira-peer/src/prime_port.rs`: `TransportClass`, `PORT_SELECT_VERSION`,
  `preferred_port` / `preferred_port_index`, `next_candidate_port*`,
  `select_available_port`, identity-aware loopback bind helpers
- Unit tests: stability, collision → next, finite full wrap
- Living smoke / QUEUE advance to first OPEN `#234`
- RFC-D this file

## 5. Non-Goals

```text
Presence Record (#234)
RendezvousProvider / EVM (#235+)
CLI peer port command (#243)
Desktop UX port panel (#244)
changing P_AIRA membership rules (#232)
HTTP / STUN / Polygon RPC
RFC-0123 consolidating body (#247)
```

## 6. Algorithm

```text
version = "aira:port-select:v1"
class ∈ { "tcp-peer", "udp-discv" }
preimage = identity_ref || class || version   # UTF-8 byte concatenation
index = u64_be(SHA-256(preimage)[0..8]) mod 1491
preferred = P_AIRA[index]
available: try index, index+1, … wrap; stop after 1491 attempts
```

## 7. Compatibility Impact

Additive API. Existing fail-closed validation unchanged. Desktop default bind may remain `49157` until CLI/Desktop atoms wire identity-aware selection.

## 8. Security Impact

Selection is not authentication. Hash only maps identity strings to ports.

## 9. Failure Semantics

Full `P_AIRA` walk with no free port → `PeerError::InvalidPort` (finite; no infinite loop).

## 10. Rollout

QUEUE `#233` → Analyze-268 → PR; next `#234` Presence Record.
