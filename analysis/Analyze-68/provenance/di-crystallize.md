# DI crystallize — Analyze-68 / QUEUE #33

## In scope (A + A1 + B1 + C1 + D1 + F1)
1. AIRA-native signed UDP FIND + NODES (not Ethereum discv5/ENR).
2. FIND target = UDP `addr` from `dht.json` (same advertised host:port).
3. `peer discv listen` multiplexes announce (store) and FIND (reply NODES = local XOR closest k).
4. Iterative client: α=3, k=8, hop cap; merge trusted-not-revoked records into `dht.json` (`source=udp`); no apply-book.
5. CLI `peer discv find --key-ref` (optional `--to` seed). Existing `dht find --apply-book` unchanged.
6. Loopback tests; requester/responder must be trusted.

## Out
Federation; Ethereum discv5; auto apply-book; announce schema/`udp_addr` field; TCP dht/dial changes.

## Decision boundaries (agent-owned)
k=8, α=3, hop cap 8; domains `aira:peer:discv:v1:find` / `:nodes`; nonce+created_at; identity_id==key_ref.
