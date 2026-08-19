# Analyze-67 — UDP discv5-style announce (QUEUE #32)

## Scope
AIRA-native signed UDP announce → local `peers/dht.json` (`source=udp`). Not Ethereum discv5. No FIND_NODE. No apply-book.

## Done when
announce+store tests green; untrusted/revoked fail-closed; docs path.
