# Peer links (Analyze-32 P0 + Analyze-33 CLI)

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

## CLI (Analyze-33)

```bash
# both nodes: init + identity create + mutual identity trust add
cargo run -p aira-cli -- --root "$A" peer add --key-ref aira:identity:bob --addr 127.0.0.1:7900
cargo run -p aira-cli -- --root "$A" peer list

# terminal B
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0
# note printed "listening …" addr; upsert it on A via peer add

# terminal A
cargo run -p aira-cli -- --root "$A" peer dial --key-ref aira:identity:bob
cargo run -p aira-cli -- --root "$A" peer send --key-ref aira:identity:bob --text "hello"
```

`peer add` fail-closed if the identity is missing from trust or revoked.

## Out of scope (later)

Noise XX / mTLS; NAT traversal; gossip; DHT; federation join; public HTTP bind; trust-delta over wire; long-running listen daemon.

## Relation to crypto docs

Peer **trust rotate** remains `identity trust rotate`. Node **secret rotate** remains `identity rotate`. Peer link hello uses current trusted verifying keys only.
