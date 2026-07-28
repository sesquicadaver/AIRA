# Peer links (Analyze-32…44)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = Ed25519 node identity (`identity/local.ed25519`)
- **Noise static** = X25519 at `identity/local.x25519` (auto-created)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Hello v1** = mutual Ed25519 (`aira:peer:hello:v1`) signing identity + `x25519_pub_hex`
- **Noise** = `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); remote static must match hello
- **Payload** = Book II `ProtocolEnvelope` inside **Noise-encrypted** length-prefixed frames
- **Trust-delta** = `peer.trust.delta` (`aira:peer:trust-delta:v1`) — revoke / rotate / unrevoke / **rekey**; apply only with `--apply-trust`
- **Gossip** (Analyze-43, opt-in `--gossip`) = forward the **original** signed trust-delta once per `message_id` (`peers/gossip_seen.json`); receivers verify originator signature (issuer may ≠ TCP peer)
- **Discovery journal** = observational `.aira/peers/discovery.json` (`last_seen`, `learned_from`, `source`); **not** the dial source
- **Relay hub** (Analyze-44, `--relay`) = live session registry; `peer.relay.deliver` couriers an original signed inner envelope to a registered peer; address-book optional `via` selects the courier
- **Rekey notify** = announce upcoming same-id pubkey **before** local cutover (`identity rotate --notify-peers`)
- **Deadlines** = 15s on dial/handshake/Noise/frame I/O; TCP accept wait unbounded (daemon)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — no DHT, no global registry. Dial targets come from the address book only (`via` redirects send through a relay).

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint` (`via` optional)
- `listen` / `accept` / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope, send_relayed_envelope, recv_envelope_allow_relayed}`
- `TrustDelta` / `make_trust_delta_envelope` / `parse_trust_delta` / `apply_trust_delta`
- `gossip_forward_trust_delta` / `GossipSeenLog` / `PeerDiscoveryStore`
- `RelayHub` / `serve_relay_peer` / `send_envelope_to_peer` / `make_relay_deliver_envelope`
- `notify_peers_of_rekey` / `notify_peer_of_rekey` (best-effort)

## CLI

```bash
# Relay node
cargo run -p aira-cli -- --root "$R" peer listen --bind 127.0.0.1:0 --relay
# NAT peer holds outbound registration
cargo run -p aira-cli -- --root "$C" peer relay-hold --key-ref aira:identity:relay --apply-trust
# Sender routes via relay
cargo run -p aira-cli -- --root "$A" peer add --key-ref aira:identity:carol --addr 127.0.0.1:1 --via aira:identity:relay
cargo run -p aira-cli -- --root "$A" peer trust-send --key-ref aira:identity:carol --op revoke --subject aira:identity:x --reason via-relay

cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --recv --apply-trust --gossip
cargo run -p aira-cli -- --root "$B" peer discovery
```

| Flag / command | Meaning |
|----------------|---------|
| `--via` on `peer add` | Courier relay identity for this peer |
| `--relay` on listen | Hub mode (register + forward deliver) |
| `peer relay-hold` | Outbound session to relay; receive courier inners |
| `--gossip` | Fanout trust-delta after apply |

## Out of scope (later)

STUN/ICE hole punching; DHT; federation join; public HTTP bind; remote TrustStore dual-key; mTLS; durable relay session store.

See `Analyze-43/provenance/ADR-connectivity-relay-first.md`.

## Relation to crypto docs

Peer **trust rotate** remains `identity trust rotate`. Node **secret rotate** remains `identity rotate` (Ed25519). Noise static is separate (`local.x25519`). Trust-delta / rekey over peer is an **announcement** applied only when the listener opts in with `--apply-trust`. Rekey notify must precede cutover. Gossip/relay preserve originator signature; apply still requires the originator in local trust.
