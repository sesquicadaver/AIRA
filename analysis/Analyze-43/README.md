# Analyze-43 — Gossip micro + discovery persist

**Scope:** One-hop gossip fanout for `peer.trust.delta` (dedupe by `message_id`) + durable `.aira/peers/discovery.json`. ADR: relay-first NAT next, DHT later. No STUN/DHT impl. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## Ralplan (APPROVED — consensus)

### Principles
1. Forward the **original** signed envelope (issuer unchanged)
2. Each node relays a given `message_id` at most once
3. Dial targets still come from address book; discovery is observational memory
4. Gossip is opt-in (`--gossip` with `--apply-trust`)
5. Document next steps: relay-first NAT, then DHT

### Decision Drivers
1. User chose synthesis option 1 (A+B + ADR)
2. Existing Noise + trust-delta + address book
3. QUEUE anti-pattern: no NAT+DHT+gossip in one PR

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Exact-envelope flood + message_id seen** | No payload mutation; simple | Needs originator in receiver trust |
| B. Hop field inside TrustDelta | Explicit TTL | Breaks signature on decrement |
| C. Re-sign as forwarder | Changes issuer semantics | Wrong for trust provenance |

**Chosen: A.**

### ADR (next)
- **Relay-first NAT** before hole punching
- **DHT** only after discovery persist + gossip prove useful

### Acceptance
- [x] A→B→C trust-delta gossip with dedupe
- [x] `discovery.json` updated on gossip/direct sighting
- [x] CLI `peer discovery` + listen `--gossip`
- [x] docs/peer-link.md + QUEUE #10 split/DONE for this micro
- [x] Tests + clippy

## Delivered

- `send_relayed_trust_delta` / `recv_envelope_allow_relayed_trust_delta`
- `gossip_forward_trust_delta` + `peers/gossip_seen.json`
- `PeerDiscoveryStore` → `peers/discovery.json`
- CLI: `--gossip`, `peer discovery`
