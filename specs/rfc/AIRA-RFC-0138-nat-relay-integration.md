# AIRA-RFC-0138 — NAT/relay inbound-blocked integration (RFC-D)

## 1. Summary

Phase N `#246`: when both peers have inbound blocked, they use a trusted relay courier path — direct dial fails, Noise sessions to the hub succeed, and signed peer payloads are delivered via `RelayHub`. RFC-0123 consolidating body is `#247`.

## 2. Problem Statement

TZ §48 requires an acceptance test for dual inbound-blocked nodes that still establish encrypted peer communication through a relay.

## 3. Motivation

Prove dial planner + existing courier semantics under inbound-blocked AddressBook configuration without inventing a TCP proxy.

## 4. Scope

- `aira-peer::nat_relay` — `configure_inbound_blocked_via_relay`, `plan_inbound_blocked_relay_path`
- Integration test: empty ledger substrate, dual blocked directs, hub Noise + trust-delta courier
- RFC-D this file; QUEUE → `#247`

## 5. Non-Goals

```text
RFC-0123 consolidating body (#247)
TCP-level Noise tunnel through relay (courier model retained)
Auto-trust from ledger
```

## 6. Compatibility / Security

Additive. Hubs never verify inner payloads. `DISCOVERED ≠ TRUSTED`. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#246` → Analyze-281 → PR; next `#247` RFC-0123 close.
