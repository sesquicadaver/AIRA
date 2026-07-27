# Peer links (Analyze-32 P0)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = existing Ed25519 node identity (`identity/local.ed25519`)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Transport** = length-prefixed TCP frames (u32 BE) on an explicit bind (tests use `127.0.0.1`)
- **Hello** = mutual Ed25519 challenge/response (`aira:peer:hello:v0`)
- **Payload** = Book II `ProtocolEnvelope` (signed); issuer must equal authenticated peer; verify is **strict** over `payload_hash` (no local-test domain fallback on the wire)
- **Deadlines** = 10s default timeout on dial/accept/handshake/frame I/O
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — no DHT, no global registry.

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint`
- `listen` / `accept` / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope}`

## Out of scope (later)

Noise XX / mTLS; NAT traversal; gossip; DHT; federation join; public HTTP bind; CLI `peer send`.

## Relation to crypto docs

Peer **trust rotate** remains `identity trust rotate`. Node **secret rotate** remains `identity rotate`. Peer link hello uses current trusted verifying keys only.
