# Peer links (Analyze-32…59)

Decentralized node-to-node messaging **without a controlling center**.

## Model

- **Identity** = Ed25519 node identity (`identity/local.ed25519`)
- **Noise static** = X25519 at `identity/local.x25519` (auto-created; rotated with `identity rotate`, Analyze-49)
- **Admission** = local `.aira/identity/trust.json` (+ CRL); unknown/revoked peers are rejected
- **Hello v1** = mutual Ed25519 (`aira:peer:hello:v1`) signing identity + `x25519_pub_hex`
- **Noise** = `Noise_XX_25519_ChaChaPoly_BLAKE2s` (`snow`); remote static must match hello
- **Payload** = Book II `ProtocolEnvelope` inside **Noise-encrypted** length-prefixed frames
- **Envelope signature** (SEC-2 / QUEUE #135) = canonical descriptor hash (full envelope, not `payload_hash` bytes alone); `signature.key_ref` must equal `issuer_identity`; peer `recv_envelope` verifies via trust keyring
- **Envelope freshness** (QUEUE `#194` / RFC-0092) = after signature, reject expired `expires_at`, `|created_at − now|` beyond 300s skew, or a `message_id` already seen in the 600s replay window (`peers/envelope_replay.json`)
- **Trust-delta** = `peer.trust.delta` (`aira:peer:trust-delta:v1`) — revoke / rotate / unrevoke / **rekey**; apply only with `--apply-trust`
- **Self-sovereign apply** (Analyze-52) = `subject_id` must equal envelope `issuer` for every op; third-party CRL stays on local CLI `trust revoke` / `trust rotate`
- **Gossip** (Analyze-43/53, opt-in `--gossip`) = forward the **original** signed trust-delta once per `message_id` (`peers/gossip_seen.json`); skip forward when `subject_id ≠ issuer` (mark-seen); apply still enforces issuer==subject
- **Discovery journal** = observational `.aira/peers/discovery.json`; **not** the dial source
- **Relay hub** (Analyze-44/58, `--relay`) = live session registry + durable `peers/relay_hub.json` membership; `peer.relay.deliver` + address-book `via`; optional `--relay-ttl-days` prunes stale **offline** rows
- **DHT-lite** (Analyze-47) = durable `.aira/peers/dht.json` with XOR closest ranking; `peer.dht.announce` over authenticated links (trusted mesh only — not UDP/discv5)
- **Rekey notify** = announce upcoming same-id pubkey **before** local cutover
- **Deadlines** = 15s on dial/handshake/Noise/frame I/O; TCP accept wait unbounded (daemon)
- **Listen** = loopback-only via `listen`; non-loopback requires `listen_explicit`
- **Daemon accept** (Analyze-59) = `accept_tcp` on the accept loop; hello+Noise (`complete_accept`) and optional `--recv` / `--relay` session work run on per-connection tasks so a hung handshake cannot block further TCP accepts. `--once` stays sequential. Discovery / “accepted” / relay register happen only after successful handshake.

Static address book: `.aira/peers/address_book.json` — authoritative dial source. DHT is advisory lookup memory. Opt-in `--apply-book` (Analyze-57) promotes exact DHT hits / inbound announces into the book without changing the default.

## Crate

`aira-peer`:

- `AddressBook` / `PeerEndpoint` (`via` optional)
- `listen` / `accept_tcp` / `complete_accept` / `accept` (composed) / `dial`
- `AuthenticatedPeer::{send_envelope, recv_envelope, send_relayed_envelope, recv_envelope_allow_relayed}`
- `TrustDelta` / gossip / discovery / relay hub APIs
- `PeerDhtStore` / `dht_announce_to_peers` / `apply_dht_announce`
- `bind_udp` / `send_discv_announce` / `apply_discv_announce` (Analyze-67)
- `notify_peers_of_rekey` / `notify_peer_of_rekey`

## CLI

```bash
cargo run -p aira-cli -- --root "$B" peer listen --bind 127.0.0.1:49157 --recv --dht --apply-book
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

Канон черги: [`QUEUE.md`](../QUEUE.md). Phase C: [`docs/phase-c-plan.md`](phase-c-plan.md).

Shipped (не Out): mTLS require (`--tls-client-ca`, A-51); DHT-lite (A-47); DHT→address_book `--apply-book` (A-57 / #22); relay hub (A-44); durable relay registry (A-58 / #23); gossip (A-43); gossip self-sovereign forward filter (A-53 / #18); concurrent accept (`accept_tcp` + spawned `complete_accept`, A-59 / #24); systemd examples (`deploy/systemd/`, [runbook-systemd.md](runbook-systemd.md), A-60 / #25); **STUN Binding reflexive** (A-66 / #31); **UDP discv announce** (A-67 / #32); **iterative FIND_NODE** (A-68 / #33); **federation join pin** (A-70 / #35); **tenant `.prev` prune** (A-71 / #36); **tenant `--secret-hex-file`** (A-72 / #37).

## Federation join (Analyze-70)

Local operator ceremony — **not** Book II Join Request/Response and **not** a peer handshake. A self-signed descriptor is verified against its embedded pubkey, then `identity_ref` is pinned in `trust.json` and `.aira/federation/membership.json` is written. Other federation members stay Untrusted until separately `identity trust add`. One membership; a different `federation_id` is fail-closed (`leave` is later).

```bash
cargo run -p aira-cli -- --root "$ROOT" federation join --descriptor ./fed.json
```

Descriptor `schema` = `aira:federation:descriptor:v1`. Canonical bytes (no signature):
`aira:federation:descriptor:v1|{schema}|{federation_id}|{federation_type}|{identity_ref}|{public_key_hex}`.

## STUN Binding (Analyze-66)

Discover a reflexive `IP:port` via RFC 5389 Binding (not full ICE). **`dial` is unchanged** — still TCP to `address_book.json`.

```bash
# required server — no public default (also AIRA_STUN_SERVER)
cargo run -p aira-cli -- --root "$A" peer stun query --stun-server 127.0.0.1:3478
# writes peers/stun_reflexive.json
cargo run -p aira-cli -- --root "$A" peer dht announce --from-stun
# fail-closed if both:
# peer dht announce --addr 1.2.3.4:9 --from-stun
```

Path: `stun query` → `stun_reflexive.json` → `dht announce --from-stun` → remote `--apply-book` / static book → `peer dial`.

**Out of this slice:** ICE connectivity-check; UDP peer sessions; TURN; STUN-per-dial; default public STUN; upsert into address book.

## UDP discv announce (Analyze-67)

Local one-hop signed UDP datagram (not Ethereum discv5/ENR). Receiver upserts `peers/dht.json` with `source=udp` if the issuer is trusted and not revoked. **Does not** change `dial` or auto-promote the address book.

## Iterative FIND_NODE (Analyze-68)

`peer discv listen` multiplexes announce + FIND. FIND is a signed UDP query; responder replies NODES = local XOR-closest `k` from `dht.json`. Client iterates α=3 / hop cap 8. FIND is sent to the **same host:port** as advertised `addr` (UDP). Returned hints merge only if the identity is already in TrustStore (not revoked); `source=udp:nodes:<responder>`. No apply-book — still `peer dht find --apply-book` for exact hits.

```bash
cargo run -p aira-cli -- --root "$B" peer discv listen --bind 127.0.0.1:PORT
cargo run -p aira-cli -- --root "$A" peer discv find --key-ref aira:identity:carol --to 127.0.0.1:PORT
```

Untrusted requester / bad signature → no NODES. TCP `dial` unchanged.

```bash
cargo run -p aira-cli -- --root "$B" peer discv listen --bind 127.0.0.1:49157 --once
cargo run -p aira-cli -- --root "$A" peer discv announce --to 127.0.0.1:PORT --addr 127.0.0.1:7900
# or --from-stun after `peer stun query`
cargo run -p aira-cli -- --root "$B" peer dht find --key-ref aira:identity:alice --apply-book
```

Non-loopback UDP bind requires `--explicit`. Untrusted / revoked / bad signature → drop, no store.

**WONT-NEED:** dedicated x25519 peer-notify after rotate (Analyze-54 / QUEUE #19) — hello v1 already Ed25519-binds `x25519_pub_hex` each dial; Noise remote static is checked against that hello. No durable remote Noise-static cache.

See [`analysis/Analyze-43/provenance/ADR-connectivity-relay-first.md`](../analysis/Analyze-43/provenance/ADR-connectivity-relay-first.md).

Desktop supervise mapping (P0–P6): [`desktop-network-profiles.md`](desktop-network-profiles.md).


## Preferred port (Phase N `#233`)

Deterministic selection: `aira_peer::preferred_port(identity, TransportClass)` over `P_AIRA` (RFC-0125). Collision walks the next primes with wrap.


## Presence Record (Phase N `#234`)

`NodePresenceRecord` (`aira:schema:peer:presence-record:0.1`) — canonical Ed25519; see RFC-0126.


## RendezvousProvider (Phase N `#235`)

Ledger-agnostic `aira_peer::RendezvousProvider` + `MockRendezvousProvider` (RFC-0127). EVM adapter: `#236`. Discovery does not upsert TrustStore.


## EvmRendezvousProvider (Phase N `#236`)

`EvmRendezvousProvider` + local ledger double; Amoy `80002` / Polygon `137` config hooks (RFC-0128). EVM payer ≠ AIRA identity.


## Rendezvous publish/query (Phase N `#237`)

`RendezvousClient` enforces TTL/sequence/size/query caps; local `peers/rendezvous.json` (RFC-0129).


## Reachability Probe (Phase N `#238`)

Peer-assisted signed challenge + attestation; hairpin forbidden (RFC-0130).


## Reachability states (Phase N `#239`)

`ReachabilityLocalState` persists UNKNOWN…OFFLINE in `peers/reachability.json` (RFC-0131). DIRECT only after verified probe.


## AddressBook promotion (Phase N `#240`)

`promote_presence_to_address_book`: valid Presence + trust policy → dial book only; no auto-trust (RFC-0132).

## Relay integration (Phase N `#241`)

`plan_dial_path` / `RelayAdvertisement` / `select_relay_reservations`: direct→NAT→relay order; prime-port ads; dual reservation SHOULD; no auto-trust (RFC-0133). Live NAT/relay Noise smoke is `#246`.

## Presence refresh (Phase N `#242`)

`refresh_and_sign_presence` / `endpoint_change_and_sign_presence` / `retain_unexpired_presence`: sequence++; renew TTL; endpoint change drops old ads; notify list from AddressBook (RFC-0134). CLI is `#243`.
