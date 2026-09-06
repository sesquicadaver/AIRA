# AIRA-RFC-0140 — Live EVM rendezvous JSON-RPC (RFC-D)

## 1. Summary

Phase N-fix `#248`: `EvmRendezvousProvider` with `use_local_double=false` dials real HTTP(S) JSON-RPC for publish/query (local anvil / `ReferenceEvmRendezvousRpc` stand-in, or documented Amoy URL). No stub-error on live config. RFC-0139 stays file-free until `#254`.

## 2. Problem Statement

`#236`/`#237` left Amoy/mainnet live dial fail-closed. Post-N audit: “global rendezvous” without JSON-RPC I/O is dishonest.

## 3. Motivation

Prove a real socket path for EVM-shaped rendezvous in CI without requiring live Polygon mainnet. Presence authenticity remains AIRA Ed25519; EVM payer ≠ identity.

## 4. Scope

- `json_rpc_http` — ureq JSON-RPC 2.0 client
- `evm_rendezvous_rpc` — reference HTTP surface (`aira_rendezvous_*` + `eth_chainId`)
- `EvmRendezvousConfig::anvil_live` / `amoy_live`; live backend on `EvmRendezvousProvider`
- Tests via `ReferenceEvmRendezvousRpc` (no public Amoy required in CI)
- RFC-D this file; QUEUE → `#249`

## 5. Non-Goals

```text
Two-process ab ovo (#249)
On-chain Solidity contract deployment as CI required
aira-core ledger deps
RFC-0139 consolidating body (#254)
```

## 6. Compatibility / Security

Additive. `DISCOVERED ≠ TRUSTED`. Live path verifies `eth_chainId` against config. Contract address checked on reference RPC.

## 7. Rollout

QUEUE `#248` → Analyze-283 → PR; next `#249` real ab ovo harness.
