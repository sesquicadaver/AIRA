# AIRA-RFC-0132 — AddressBook promotion from Presence (RFC-D)

## 1. Summary

Phase N `#240`: promote a **valid** Presence into `peers/address_book.json` only when local trust policy allows the peer. Never auto-upsert TrustStore (`DISCOVERED ≠ TRUSTED`). AddressBook remains the dial authority. Relay product glue is `#241`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Rendezvous discovery alone must not write dial endpoints. Without trust-gated promotion, ledger peers would become dialable without operator trust.

## 3. Motivation

TZ §14: Presence valid AND trust policy → AddressBook. Ledger/DHT/discovery must not replace AddressBook.

## 4. Scope

- `aira-peer::presence_promote` — `promote_presence_to_address_book`, `trust_policy_allows`, `dial_target_from_presence`
- Prefer `tcp-peer` direct; else first relay (`via`)
- Unit tests (trusted promote, untrusted no auto-trust, revoked reject, relay via)
- RFC-D this file; QUEUE advance to `#241`

## 5. Non-Goals

```text
Relay integration product path (#241)
Presence refresh (#242)
CLI (#243)
Auto TrustStore upsert from ledger
RFC-0123 consolidating body (#247)
```

## 6. Compatibility / Security

Additive. Fail-closed on untrusted/revoked. Does not add ledger deps to `aira-core`.

## 7. Rollout

QUEUE `#240` → Analyze-275 → PR; next `#241` Relay integration.
