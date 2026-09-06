# AIRA-RFC-0144 — Inbound firewall / netns honesty (RFC-D)

## 1. Summary

Phase N-fix `#252`: distinguish **non-listening placeholder** (`#246` courier) from **listening + firewall INPUT DROP**. Docker/`NET_ADMIN` smoke proves the latter; CI without Docker takes the documented script path. RFC-0139 stays file-free until `#254`.

## 2. Problem Statement

`#246` labeled “inbound blocked” but used a non-listening AddressBook slot. That is not NAT/firewall semantics.

## 3. Motivation

Operators and audits must not confuse “nothing listens” with “listens but DROP”. Relay courier remains valid under the honest placeholder name.

## 4. Scope

- `configure_non_listening_placeholder_via_relay` + semantics constant; historical alias retained
- `inbound_firewall` helpers + `scripts/inbound_firewall_smoke.sh` (Docker + iptables)
- Tests: kinds differ; listening ≠ placeholder; docker-or-documented integration
- Relay courier test kept under honest naming
- RFC-D this file; QUEUE → `#253`

## 5. Non-Goals

```text
next_candidate_port wrap (#253)
Presence created_at < expires_at / RFC-0139 (#254)
Full CGNAT field trial as required CI
Rewriting #246 courier as iptables
```

## 6. Compatibility / Security

Additive. `DISCOVERED ≠ TRUSTED`. No ledger deps in `aira-core`. Firewall smoke is opt-in capability (Docker `NET_ADMIN`).

## 7. Rollout

QUEUE `#252` → Analyze-287 → PR; next `#253` port wrap.
