# Ralplan — Analyze-67 / QUEUE #32

## Principles
1. One hop: UDP announce → local store. No iterative FIND_NODE.
2. Fail-closed: untrusted/revoked/bad sig/bad bind → drop with error, no store.
3. TCP mesh DHT path stays; UDP is an additional ingress into the same table.
4. Tests loopback-only.

## Implementation
1. **`aira-peer::discv`**: datagram schema, sign/verify (Ed25519 over domain || canonical JSON), `listen_udp` loopback / `listen_udp_explicit`, `announce_udp`, `recv_one_and_store`.
2. **Store:** `PeerDhtStore::upsert(..., Some("udp"))` after trust+sig.
3. **CLI:** `PeerDiscvCommands::{Listen, Announce}`; announce addr via existing `resolve_dht_announce_addr`.
4. **Docs** `docs/peer-link.md` + Living Spec + QUEUE on ship.
5. **Tests:** roundtrip A→B store; untrusted rejected; revoked rejected; loopback bind rejects non-loopback without explicit; TCP dial unchanged.

## Out
Same as DI crystallize Out.

## Done when
announce+store green; FIND_NODE not present.
