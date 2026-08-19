# Analyze-47 — Trusted-mesh DHT-lite

**Scope:** Local durable DHT table + XOR closest + `peer.dht.announce` over authenticated peer links. No UDP/discv5/STUN. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## RALPLAN-DR

### Principles
1. DHT is advisory memory; address book remains authoritative dial source unless operator upserts
2. Announce preserves originator signature (issuer = announcer)
3. XOR distance over sha256(identity_id) — Kademlia-lite ranking only
4. Trusted peers only
5. No public global DHT

### Decision Drivers
1. QUEUE + ADR: DHT after gossip/relay
2. Existing Noise + address book + discovery
3. Anti-pattern: no STUN+DHT in one PR

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Local table + announce + XOR find** | Small, testable | Not iterative FIND_NODE |
| B. Full discv5/UDP | Standard | Huge, wrong stage |
| C. Skip DHT → mTLS | Useful | Ignores QUEUE/ADR order |

**Chosen: A.**

### Architect
- **Antithesis:** Enrich discovery.json vs separate dht.json — separate keeps observational vs ranked lookup clear.
- **Tension:** Auto-upsert address book from DHT vs manual — synthesis: find prints; optional `--apply-book` later Out.
- **Critic: APPROVE**

### Acceptance
- [x] `PeerDhtStore` persist + XOR closest
- [x] `peer.dht.announce` make/parse/apply
- [x] CLI dht announce/find/list + listen `--dht`
- [x] Integration: A announces to B; B finds A
- [x] docs + QUEUE; tests + clippy; APPROVE/CLEAR

### Delivered
- `peers/dht.json` + XOR closest
- `peer.dht.announce` + CLI
- Anti-spoof apply (issuer must equal announced identity)

### Out
discv5, UDP, iterative multi-hop FIND_NODE, STUN, auto address-book mutation.
