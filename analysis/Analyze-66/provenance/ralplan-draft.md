# Ralplan — Analyze-66 / QUEUE #31

## Principles
1. One mechanism: Binding client only (not ICE).
2. Fail-closed: no server URL → error; `--from-stun` + `--addr` → error.
3. Dial path documentation without changing `dial()` TCP semantics.
4. Tests use in-process mock; venv/local only.

## Implementation plan
1. **`aira-peer::stun`**: build Binding request; send UDP; parse success XOR-MAPPED-ADDRESS; `StunReflexiveRecord` load/save under `peers/stun_reflexive.json`.
2. **Mock**: `stun::mock_server` (or test-only) answering Binding with configured mapped addr.
3. **API**: `query_stun_reflexive(server, timeout) -> SocketAddr`; `save_stun_reflexive(root, record)`; `load_stun_reflexive(root)`.
4. **DHT**: `dht_announce` path accepts addr from reflexive file when flag set (library helper + CLI).
5. **CLI** (`aira-cli` peer):
   - `stun query --stun-server` [env `AIRA_STUN_SERVER`]
   - `dht announce --from-stun` (mutually exclusive with `--addr`)
6. **Docs** + Living Spec + QUEUE on ship.
7. **Tests**: mock roundtrip; persist file; `--from-stun` helper; conflict fail-closed; dial still book-only (existing tests).

## Out
Same as DI crystallize Out.

## Done when
Documented path + green tests with mock STUN; no discv5.
