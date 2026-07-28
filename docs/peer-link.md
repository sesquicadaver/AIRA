# Peer links (Analyze-32…36)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = Ed25519 node identity (`identity/local.ed25519`)
- **Noise static** = X25519 at `identity/local.x25519` (auto-created)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Hello v1** = mutual Ed25519 (`aira:peer:hello:v1`) signing identity + `x25519_pub_hex`
- **Noise** = `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); remote static must match hello
- **Payload** = Book II `ProtocolEnvelope` inside **Noise-encrypted** length-prefixed frames
- **Trust-delta** = `peer.trust.delta` (`aira:peer:trust-delta:v1`) — revoke / rotate / unrevoke; apply only with `--apply-trust`
- **Deadlines** = 15s on dial/handshake/Noise/frame I/O; TCP accept wait unbounded (daemon)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — no DHT, no global registry.

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint`
- `listen` / `accept` / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope}` (encrypted)
- `TrustDelta` / `make_trust_delta_envelope` / `parse_trust_delta` / `apply_trust_delta`

## CLI

```bash
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --recv --apply-trust
cargo run -p aira-cli -- --root "$A" peer dial --key-ref aira:identity:bob
cargo run -p aira-cli -- --root "$A" peer send --key-ref aira:identity:bob --text "hello"
cargo run -p aira-cli -- --root "$A" peer trust-send \
  --key-ref aira:identity:bob --op revoke --subject aira:identity:carol --reason compromised
# one-shot + recv:
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --once --recv
```

| Flag | Default | Meaning |
|------|---------|---------|
| `--bind` | `127.0.0.1:0` | loopback bind |
| `--once` | off | exit after one accept |
| `--recv` | off | receive one envelope after hello (daemon: async task) |
| `--apply-trust` | off | apply `peer.trust.delta` into local `trust.json` (requires `--recv`) |

## Out of scope (later)

NAT traversal; gossip; DHT; federation join; public HTTP bind; auto-notify on local rotate; mTLS.

## Relation to crypto docs

Peer **trust rotate** remains `identity trust rotate`. Node **secret rotate** remains `identity rotate` (Ed25519). Noise static is separate (`local.x25519`) and regenerated only if file missing (not rotated with Ed25519 in this slice). Trust-delta over peer is an **announcement** applied only when the listener opts in with `--apply-trust`.
