# Peer links (Analyze-32…38)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = Ed25519 node identity (`identity/local.ed25519`)
- **Noise static** = X25519 at `identity/local.x25519` (auto-created)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Hello v1** = mutual Ed25519 (`aira:peer:hello:v1`) signing identity + `x25519_pub_hex`
- **Noise** = `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); remote static must match hello
- **Payload** = Book II `ProtocolEnvelope` inside **Noise-encrypted** length-prefixed frames
- **Trust-delta** = `peer.trust.delta` (`aira:peer:trust-delta:v1`) — revoke / rotate / unrevoke / **rekey**; apply only with `--apply-trust`
- **Rekey notify** = announce upcoming same-id pubkey **before** local cutover (`identity rotate --notify-peers`)
- **Deadlines** = 15s on dial/handshake/Noise/frame I/O; TCP accept wait unbounded (daemon)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — no DHT, no global registry.

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint`
- `listen` / `accept` / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope}` (encrypted)
- `TrustDelta` / `make_trust_delta_envelope` / `parse_trust_delta` / `apply_trust_delta`
- `notify_peers_of_rekey` / `notify_peer_of_rekey` (best-effort)

## CLI

```bash
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --recv --apply-trust
cargo run -p aira-cli -- --root "$A" peer dial --key-ref aira:identity:bob
cargo run -p aira-cli -- --root "$A" peer send --key-ref aira:identity:bob --text "hello"
cargo run -p aira-cli -- --root "$A" peer trust-send \
  --key-ref aira:identity:bob --op revoke --subject aira:identity:carol --reason compromised
# notify then rotate (order matters — hello must still verify):
cargo run -p aira-cli -- --root "$A" identity rotate --notify-peers
cargo run -p aira-cli -- --root "$A" peer notify-rekey --pubkey-hex <64-hex>
```

| Flag | Default | Meaning |
|------|---------|---------|
| `--bind` | `127.0.0.1:0` | loopback bind |
| `--once` | off | exit after one accept |
| `--recv` | off | receive one envelope after hello (daemon: async task) |
| `--apply-trust` | off | apply `peer.trust.delta` into local `trust.json` (requires `--recv`) |

## Out of scope (later)

NAT traversal; gossip; DHT; federation join; public HTTP bind; remote TrustStore dual-key; mTLS.

## Relation to crypto docs

Peer **trust rotate** remains `identity trust rotate`. Node **secret rotate** remains `identity rotate` (Ed25519). Noise static is separate (`local.x25519`). Trust-delta / rekey over peer is an **announcement** applied only when the listener opts in with `--apply-trust`. Rekey notify must precede cutover.
