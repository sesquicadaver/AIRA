# Analyze-192 — session durable events (QUEUE #157)

## Status
DONE — `LocalSession` / `init_node` wire `FileChainEventLog`.

## Done when
`init_node` creates `events/file-chain-log.json`; submit appends; reopen + `event_tail` roundtrip; `session_durable_file_chain_roundtrip` green.

## Out
Sqlite object path docs/tests (#158); remove legacy JSON dual-write.
