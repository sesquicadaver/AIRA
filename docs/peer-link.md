# Peer links (Analyze-32 P0 + Analyze-33 CLI + Analyze-34 daemon)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = existing Ed25519 node identity (`identity/local.ed25519`)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Transport** = length-prefixed TCP frames (u32 BE) on an explicit bind (tests use `127.0.0.1`)
- **Hello** = mutual Ed25519 challenge/response (`aira:peer:hello:v0`)
- **Payload** = Book II `ProtocolEnvelope` (signed); issuer must equal authenticated peer; verify is **strict** over `payload_hash` (no local-test domain fallback on the wire)
- **Deadlines** = 10s default timeout on dial/handshake/frame I/O; **TCP accept wait is unbounded** (daemon idle)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — no DHT, no global registry.

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint`
- `listen` / `accept` / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope}`

## CLI

```bash
# both nodes: init + identity create + mutual identity trust add
cargo run -p aira-cli -- --root "$A" peer add --key-ref aira:identity:bob --addr 127.0.0.1:7900
cargo run -p aira-cli -- --root "$A" peer list

# terminal B — persistent listen (hello-only; dial smoke OK)
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0
# note printed "listening …" addr; upsert it on A via peer add

# terminal A
cargo run -p aira-cli -- --root "$A" peer dial --key-ref aira:identity:bob
cargo run -p aira-cli -- --root "$A" peer send --key-ref aira:identity:bob --text "hello"

# one-shot listen that also receives one envelope (A-33 style)
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --once --recv
```

Flags for `peer listen`:

| Flag | Default | Meaning |
|------|---------|---------|
| `--bind` | `127.0.0.1:0` | loopback bind |
| `--once` | off | exit after one accept |
| `--recv` | off | receive one envelope after hello (daemon: async task per peer) |

`peer add` fail-closed if the identity is missing from trust or revoked.

## Out of scope (later)

Noise XX / mTLS; NAT traversal; gossip; DHT; federation join; public HTTP bind; trust-delta over wire.

## Relation to crypto docs

Peer **trust rotate** remains `identity trust rotate`. Node **secret rotate** remains `identity rotate`. Peer link hello uses current trusted verifying keys only.
