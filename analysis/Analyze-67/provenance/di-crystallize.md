# DI crystallize — Analyze-67 / QUEUE #32

## In scope (A + A1 + B1 + C1 + D1 + F1)
1. AIRA-native signed UDP datagram: `identity_id` + `addr` + Ed25519, domain-tagged (not Ethereum discv5/ENR).
2. CLI `peer discv listen --bind` (loopback default; non-loopback explicit like TCP `listen_explicit`) and `peer discv announce --to host:port` (+ `--addr` / `--from-stun` reuse).
3. Receiver: verify signature; issuer must be in TrustStore and not revoked; then upsert `PeerDhtStore` (`peers/dht.json`) with `source=udp`.
4. TCP `peer.dht.announce` / `peer listen` / `dial` unchanged.
5. No auto apply-book from UDP (book still via `dht find --apply-book`).
6. In-process UDP tests (loopback); no live WAN.

## Out
FIND_NODE (#33); Ethereum discv5/ENR; TCP announce changes; STUN changes; auto apply-book; Noise/UDP peer sessions; public bind without explicit flag.

## Decision boundaries (agent-owned)
Compact JSON + domain tag `aira:peer:discv:v1:announce`; payload includes `nonce_hex` + `created_at`; `identity_id == signature.key_ref`; CLI names above; loopback bind helper; crate module under `aira-peer`.
