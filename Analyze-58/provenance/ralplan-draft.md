# Ralplan — Analyze-58 (revision 1)

## Principles
1. Registry on disk; live mpsc routes stay RAM-only
2. Keep history on disconnect (`online:false`)
3. Optional TTL prune — **only** `online:false` rows older than N days (never prune live `online:true`)
4. Fail-closed disk/schema errors surface as `Result` (no silent desync)
5. No STUN / session resurrect

## API contract (required)
- Separate durable store type `RelayHubRegistry` (load/save/prune) OR hub methods that return `Result`.
- `serve_relay_peer` / CLI open path: on register → fallible durable upsert; on unregister → fallible mark offline. IO/schema errors **propagate** (do not swallow).
- Do **not** keep infallible `register()` as the only persist hook hiding IO.

## TTL
- CLI: `peer listen --relay [--relay-ttl-days N]` — omit = no prune.
- When `Some(N)`: on load and on each durable write, drop entries where `online == false` AND `last_seen` older than N days.
- Recommended N = 31. Never TTL-delete `online:true`.

## Test-spec
1. Persist + reload after restart (new hub/registry instance)
2. Unregister → online false on disk
3. TTL Some(31) prunes **stale offline** only
4. TTL does **not** remove online:true even if last_seen old
5. TTL None keeps stale offline
6. Live deliver fails until re-register after reload
7. Corrupt/schema mismatch → Err (fail closed)

## Out
STUN; session crypto; undelivered queue; multi-hop; #24 concurrent recv
