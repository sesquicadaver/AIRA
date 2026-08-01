# Peer links (Analyze-32…53)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = Ed25519 node identity (`identity/local.ed25519`)
- **Noise static** = X25519 at `identity/local.x25519` (auto-created; rotated with `identity rotate`, Analyze-49)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Hello v1** = mutual Ed25519 (`aira:peer:hello:v1`) signing identity + `x25519_pub_hex`
- **Noise** = `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); remote static must match hello
- **Payload** = Book II `ProtocolEnvelope` inside **Noise-encrypted** length-prefixed frames
- **Trust-delta** = `peer.trust.delta` (`aira:peer:trust-delta:v1`) — revoke / rotate / unrevoke / **rekey**; apply only with `--apply-trust`
- **Self-sovereign apply** (Analyze-52) = `subject_id` must equal envelope `issuer` for every op; third-party CRL stays on local CLI `trust revoke` / `trust rotate`
- **Gossip** (Analyze-43/53, opt-in `--gossip`) = forward the **original** signed trust-delta once per `message_id` (`peers/gossip_seen.json`); skip forward when `subject_id ≠ issuer` (mark-seen); apply still enforces issuer==subject
- **Discovery journal** = observational `.aira/peers/discovery.json`; **not** the dial source
- **Relay hub** (Analyze-44/58, `--relay`) = live session registry + durable `peers/relay_hub.json` membership; `peer.relay.deliver` + address-book `via`; optional `--relay-ttl-days` prunes stale **offline** rows
- **DHT-lite** (Analyze-47) = durable `.aira/peers/dht.json` with XOR closest ranking; `peer.dht.announce` over authenticated links (trusted mesh only — not UDP/discv5)
- **Rekey notify** = announce upcoming same-id pubkey **before** local cutover
- **Deadlines** = 15s on dial/handshake/Noise/frame I/O; TCP accept wait unbounded (daemon)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`

Static address book: `.aira/peers/address_book.json` — authoritative dial source. DHT is advisory lookup memory. Opt-in `--apply-book` (Analyze-57) promotes exact DHT hits / inbound announces into the book without changing the default.

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
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:0 --recv --dht --apply-book
cargo run -p aira-cli -- --root "$A" peer dht announce --addr 127.0.0.1:7900
cargo run -p aira-cli -- --root "$B" peer dht find --key-ref aira:identity:alice --apply-book
cargo run -p aira-cli -- --root "$B" peer dht list
```

| Flag / command | Meaning |
|----------------|---------|
| `--dht` on listen | Apply inbound `peer.dht.announce` |
| `--apply-book` on listen | Requires `--dht`; also upsert announce into address book (preserves `via`) |
| `peer dht announce` | Put local addr + fan out to address book |
| `peer dht find` | Exact + XOR-closest local lookup |
| `peer dht find --apply-book` | Upsert **exact** hit into address book (closest print-only) |
| `--via` / `--relay` / `--gossip` | See Analyze-43/44 |
| `--relay-ttl-days N` | Requires `--relay`; prune offline registry rows older than N days (recommended 31) |

## Out of scope (later)

Канон черги: [`QUEUE.md`](../QUEUE.md) Phase B #24+.

Заплановано атомарно: concurrent recv (#24); systemd (#25); STUN/ICE-lite (#31); discv5 announce (#32); FIND_NODE (#33); public HTTP bind (#34); federation (#35).

Shipped (не Out): mTLS require (`--tls-client-ca`, A-51); DHT-lite (A-47); DHT→address_book `--apply-book` (A-57 / #22); relay hub (A-44); durable relay registry (A-58 / #23); gossip (A-43); gossip self-sovereign forward filter (A-53 / #18).

**WONT-NEED:** dedicated x25519 peer-notify after rotate (Analyze-54 / QUEUE #19) — hello v1 already Ed25519-binds `x25519_pub_hex` each dial; Noise remote static is checked against that hello. No durable remote Noise-static cache.

See `Analyze-43/provenance/ADR-connectivity-relay-first.md`.
