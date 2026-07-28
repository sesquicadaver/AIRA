# Peer links (Analyze-32…47)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = Ed25519 node identity (`identity/local.ed25519`)
- **Noise static** = X25519 at `identity/local.x25519` (auto-created; rotated with `identity rotate`, Analyze-49)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Hello v1** = mutual Ed25519 (`aira:peer:hello:v1`) signing identity + `x25519_pub_hex`
- **Noise** = `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); remote static must match hello
- **Payload** = Book II `ProtocolEnvelope` inside **Noise-encrypted** length-prefixed frames
- **Trust-delta** = `peer.trust.delta` (`aira:peer:trust-delta:v1`) — revoke / rotate / unrevoke / **rekey**; apply only with `--apply-trust`
- **Gossip** (Analyze-43, opt-in `--gossip`) = forward the **original** signed trust-delta once per `message_id` (`peers/gossip_seen.json`)
- **Discovery journal** = observational `.aira/peers/discovery.json`; **not** the dial source
- **Relay hub** (Analyze-44, `--relay`) = live session registry; `peer.relay.deliver` + address-book `via`
- **DHT-lite** (Analyze-47) = durable `.aira/peers/dht.json` with XOR closest ranking; `peer.dht.announce` over authenticated links (trusted mesh only — not UDP/discv5)
- **Rekey notify** = announce upcoming same-id pubkey **before** local cutover
- **Deadlines** = 15s on dial/handshake/Noise/frame I/O; TCP accept wait unbounded (daemon)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — authoritative dial source. DHT is advisory lookup memory.

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint` (`via` optional)
- `listen` / `accept` / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope, send_relayed_envelope, recv_envelope_allow_relayed}`
- `TrustDelta` / gossip / discovery / relay hub APIs
- `PeerDhtStore` / `dht_announce_to_peers` / `apply_dht_announce`
- `notify_peers_of_rekey` / `notify_peer_of_rekey`

## CLI

```bash
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --recv --dht
cargo run -p aira-cli -- --root "$A" peer dht announce --addr 127.0.0.1:7900
cargo run -p aira-cli -- --root "$B" peer dht find --key-ref aira:identity:alice
cargo run -p aira-cli -- --root "$B" peer dht list
```

| Flag / command | Meaning |
|----------------|---------|
| `--dht` on listen | Apply inbound `peer.dht.announce` |
| `peer dht announce` | Put local addr + fan out to address book |
| `peer dht find` | Exact + XOR-closest local lookup |
| `--via` / `--relay` / `--gossip` | See Analyze-43/44 |

## Out of scope (later)

STUN/ICE; UDP discv5 / iterative FIND_NODE; federation join; public HTTP bind; remote TrustStore dual-key; mTLS; durable relay session store; auto address-book mutation from DHT.

See `Analyze-43/provenance/ADR-connectivity-relay-first.md`.
