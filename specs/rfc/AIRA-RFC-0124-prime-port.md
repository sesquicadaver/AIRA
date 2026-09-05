# AIRA-RFC-0124 — Prime Private Port Invariant (RFC-D)

## 1. Summary

Phase N `#232`: AIRA-owned peer / discv / relay TCP+UDP endpoints **must** use a prime port from `P_AIRA` — the set of primes in `49152..=65535` (exactly **1491** values; first `49157`, last `65521`). Non-prime binds and address-book / DHT dial targets fail closed immediately with operator diagnostics and a suggested port. Deterministic `preferred_port(identity, …)` is **out of scope** (`#233`). Consolidating Phase N RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Without a shared structural pre-filter, peer listen defaults (`:0`, `:9797`) and ephemeral OS ports collide with service ranges and give no cheap wire-level signal that an endpoint claims AIRA transport semantics.

## 3. Motivation

Prime ports in the Dynamic/Private range are a low-cost, deterministic membership test shared by Desktop, CLI, and peer crates — not authentication, not NAT traversal, not identity binding (that is `#233`+).

## 4. Scope

- `crates/aira-peer/src/prime_port.rs` — `P_AIRA`, validate, loopback available helpers
Fail-closed hooks: `listen` / `listen_explicit` / `bind_udp` / `bind_udp_explicit`, `AddressBook` upsert/resolve, DHT announce/store promote, discv advertise + client UDP
- Desktop `DEFAULT_PEER_LISTEN=127.0.0.1:49157` + `normalize_peer_listen` prime check
- CLI peer/discv listen defaults → `127.0.0.1:49157`
- Living smoke updates in `phase_n_doc.rs` for QUEUE `#232` DONE

## 5. Non-Goals

```text
preferred_port(identity) / collision walk (#233)
Presence / EVM rendezvous (#234+)
applying prime check to HTTP node listen, STUN outbound, Polygon RPC
RFC-0123 consolidating body (#247)
changing Noise / trust / schema $id
```

## 6. Current Behavior (after `#232`)

```text
validate_aira_port / validate_aira_bind reject 0, 443, 9797, composites in-range, etc.
|P_AIRA| == 1491; binary search membership
PeerError::InvalidPort carries suggested 49157
HTTP / STUN continue to use OS ephemeral binds where appropriate
```

## 7. Proposed Change

Implemented in this atom: module + hooks + Desktop/CLI defaults + fixtures/docs.

## 8. Affected Books / Schemas / Tests

- Book / peer transport: AIRA-owned endpoints only
- Schema text: desktop settings `peer_listen` default description
- Tests: `aira-peer` `prime_port::*`, smoke/discv/dht; desktop settings/invite/lifecycle; `phase_n_doc`

## 9. Compatibility Impact

Breaking for operators who bound non-prime AIRA peer ports (`:0`, `:9797`, composites). Fail-closed with clear error + suggested `49157`. HTTP listen unchanged.

## 10. Security Impact

Structural pre-filter only. Does not replace trust, Noise, or invite auth. Reduces accidental bind to well-known service ports.

## 11. Privacy Impact

None beyond existing peer listen exposure.

## 12. Policy Impact

Desktop P1+ defaults move to `127.0.0.1:49157`.

## 13. Failure Semantics

Immediate reject on non-`P_AIRA` for AIRA-owned bind/upsert/advertise/resolve. No grace period.

## 14. Rollout

QUEUE `#232` → Analyze-267 → PR; next atom `#233` preferred selection.

## 15. Open Questions

None for this atom; identity-hashed preference deferred to `#233`.
