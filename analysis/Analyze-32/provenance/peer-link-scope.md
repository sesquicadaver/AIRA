# Provenance — Peer link P0

## Decision
New crate `aira-peer`: framed TCP + Ed25519 mutual hello + `ProtocolEnvelope`; admission via local `trust.json`; static address book.

## Why
No controlling center; reuses Book II envelope and existing identity/trust; Noise/X25519 deferred to keep one-cycle shippable MVP.

## Rejected
DHT, libp2p, gossip, Noise XX, embedding net in `aira-protocol`, public HTTP bind.

## Upstream tip
`e73bd30`
