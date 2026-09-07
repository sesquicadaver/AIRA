# AIRA-RFC-0161 — EVM/JSON-RPC honesty (RFC-D)

## 1. Summary

Phase P `#271`: declared Amoy/mainnet `https://` RPC URLs validate in live config. Dial remains PARTIAL via `http://` reference/anvil or gateway (no TLS client in `aira-peer`). `EvmLedgerClaim` labels LocalMock / ReferenceHttpJsonRpc / DeclaredHttpsConfig; **none** is an on-chain Polygon ledger success. RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

`#248` live HTTP JSON-RPC used a Mock-backed reference server; config rejected declared `https://` Amoy/mainnet defaults. Docs/tests could over-read HTTP roundtrip as ledger proof.

## 3. Motivation

`phase-p-plan` / post-O audit §7: Mock HTTP ≠ ledger; https config path for declared URLs; global live remains PARTIAL.

## 4. Scope

- `EvmRendezvousConfig` accepts `http://` and `https://` for live
- `amoy_live_declared_https` / `polygon_live_declared_https`
- `EvmLedgerClaim` + `ledger_claim()` / `is_on_chain_ledger() == false`
- HTTPS dial error names PARTIAL; docs PARTIAL
- Tests; QUEUE → `#272`

## 5. Non-Goals

```text
Lifecycle non-blocking (#272)
Full rustls Amoy dial as CI required
On-chain Solidity deployment
Consolidating RFC-0156 (#274)
```

## 6. Compatibility / Security

Additive honesty API. No Core/ledger deps. Presence still AIRA Ed25519.

## 7. Rollout

QUEUE `#271` → Analyze-306 → PR; next `#272` Lifecycle non-blocking.
