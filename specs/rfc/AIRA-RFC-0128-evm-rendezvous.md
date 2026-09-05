# AIRA-RFC-0128 — EvmRendezvousProvider local double (RFC-D)

## 1. Summary

Phase N `#236`: add `EvmRendezvousProvider` with deterministic local ledger double for CI, plus Amoy (`80002`) / Polygon mainnet (`137`) config hooks. Live JSON-RPC publish/query is `#237`. RFC-0123 stays file-free until `#247`.

## 2. Problem Statement

Trait alone cannot carry EVM profile metadata (chain_id, rpc_url, contract). Without a typed adapter + local double, CI cannot exercise EVM-shaped rendezvous without dialing Amoy.

## 3. Motivation

Blockchain = ordering/persistence/lookup substrate. AIRA Ed25519 on Presence remains authenticity. EVM tx sender is never AIRA identity.

## 4. Scope

- `aira-peer::evm_rendezvous` — `EvmRendezvousConfig`, `EvmChainProfile`, `EvmRendezvousProvider`
- Local double storage via `MockRendezvousProvider`
- Amoy/mainnet presets (`use_local_double=true`); live remote fail-closed until `#237`
- `evm_identity_hash(identity_ref)` for contract key shape
- RFC-D this file; QUEUE advance to `#237`

## 5. Non-Goals

```text
Live JSON-RPC dial (deferred; product publish/query is #237)
TTL/sequence product ledger rules (#237)
Reachability / AddressBook (#238–#240)
aira-core ledger deps
IPFS/HTTP gateway adapter
RFC-0123 consolidating body (#247)
```

## 6. Profiles

```text
LocalDouble  chain_id=31337  rpc=aira://evm-local-double
Amoy         chain_id=80002  rpc=https://rpc-amoy.polygon.technology/
Polygon      chain_id=137    rpc=https://polygon-rpc.com/
```

## 7. Compatibility / Security

Additive. `provider_kind() == "evm"`. Discovery ≠ trust.

## 8. Rollout

QUEUE `#236` → Analyze-271 → PR; next `#237` publish/query.
