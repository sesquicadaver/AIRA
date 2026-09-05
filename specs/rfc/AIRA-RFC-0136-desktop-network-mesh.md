# AIRA-RFC-0136 — Desktop Network mesh status (RFC-D)

## 1. Summary

Phase N `#244`: Desktop **Network** tab shows Identity, preferred AIRA prime port, local bind, external observed endpoint, direct/relay reachability, rendezvous connectivity, and address-book peer count, with top-level banner `DIRECT | RELAYED | OUTBOUND ONLY | OFFLINE`. Desktop only orchestrates `aira-peer` / keyring reads via `load_network_mesh_snapshot`. Ab ovo integration is `#245`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Operators need the same mesh fields as CLI (`#243`) inside Desktop without reimplementing port math or auto-trust.

## 3. Motivation

TZ §35 Network panel. Banner must not claim DIRECT from loopback-only / hairpin state (maps LOCAL_ONLY → OFFLINE).

## 4. Scope

- `aira-desktop-runtime::network_mesh` snapshot loader + unit tests
- Network tab mesh block + uk/en labels
- RFC-D this file; QUEUE → `#245`

## 5. Non-Goals

```text
Ab ovo publish→discover→trust→dial (#245)
NAT Noise integration (#246)
RFC-0123 consolidating body (#247)
Live EVM JSON-RPC dial
```

## 6. Compatibility / Security

Additive read-only UI. Discovery ≠ trust. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#244` → Analyze-279 → PR; next `#245` Ab ovo.
