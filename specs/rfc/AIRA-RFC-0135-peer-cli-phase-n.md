# AIRA-RFC-0135 — Phase N peer CLI (RFC-D)

## 1. Summary

Phase N `#243`: operator CLI `aira peer port status|select`, `reachability status|check`, `rendezvous status|publish|query`. CLI only orchestrates `aira-peer` APIs. Durable offline rendezvous uses `LocalFileRendezvousProvider` (`peers/rendezvous_ledger.json`). Primality logic stays in `prime_port` (not duplicated). Desktop Network panel is `#244`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Phase N mesh APIs need operator entry points without Desktop. Existing `listen/dial/dht/stun` remain; new commands must not reimplement port math or auto-trust.

## 3. Motivation

TZ §34 required command set. Fail-closed non-prime ports. Local bind check never claims DIRECT (hairpin forbidden).

## 4. Scope

- CLI modules: `port`, `reachability`, `rendezvous`
- `LocalFileRendezvousProvider` + `RENDEZVOUS_KIND_LOCAL_FILE`
- Clap parse tests + ledger reopen unit test
- RFC-D this file; QUEUE → `#244`

## 5. Non-Goals

```text
Desktop Network UX (#244)
Live EVM JSON-RPC dial
Ab ovo / NAT Noise integration (#245–#246)
RFC-0123 consolidating body (#247)
```

## 6. Compatibility / Security

Additive. Discovery ≠ trust. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#243` → Analyze-278 → PR; next `#244` Desktop UX.
