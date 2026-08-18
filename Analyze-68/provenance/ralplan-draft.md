# Ralplan — Analyze-68 / QUEUE #33

## Principles
1. Iterative XOR lookup over existing UDP discv path.
2. Fail-closed: untrusted/revoked/bad sig → no reply / no merge.
3. Third-party NODES records stored only if identity already in TrustStore.
4. Address book unchanged.

## Implementation
1. Extend `discv.rs`: `DiscvFind` / `DiscvNodes` signed JSON; `handle_discv_datagram` dispatch.
2. Listen: FIND → verify requester → `PeerDhtStore::closest(target, k)` → signed NODES to src.
3. `iterative_find(root, target, seeds, k, alpha, hops)` → merge + return exact/closest.
4. CLI `peer discv find --key-ref [--to] [--k]`; listen uses handle (not announce-only).
5. Docs + Living Spec + QUEUE on ship.
6. Tests: A→B→C iterative store of C; untrusted FIND dropped; untrusted NODES record skipped; announce still works on same listen.

## Out
Same as DI crystallize Out.

## Done when
closest lookup over UDP path green; no federation.
