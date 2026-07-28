# Analyze-44 — Relay-first NAT (hub)

**Scope:** Trusted relay hub with live session registration + `peer.relay.deliver` of original signed envelopes + address-book `via`. No STUN/ICE/DHT/hole punching. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## Ralplan (APPROVED — consensus)

### Principles
1. Original signature / issuer unchanged end-to-end (courier model)
2. NAT path = outbound register to trusted relay + deliver over live session
3. Dial targets: address book; `via` selects relay courier when set
4. Fail closed: relay and originator must be trusted; unknown target = error
5. No STUN/DHT in this slice

### Decision Drivers
1. ADR from A-43: relay-first before hole punching
2. User want: ETH/TRON-like reliable mesh without public bind on every node
3. QUEUE anti-pattern: do not mix DHT + NAT + gossip

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Live hub + register + deliver** | Real NAT path | In-memory sessions; more code |
| B. Dial-through only (relay dials target addr) | Simple | Fails when target is NATed |
| C. STUN/ICE first | Standard NAT | Out of ADR order |

**Chosen: A.** Invalidation: B does not satisfy “behind NAT”; C violates ADR.

### Architect (steelman / tension)
- **Antithesis:** Live hub couples listen daemon to routing state; channel mux adds complexity vs dial-through.
- **Tension:** End-to-end authenticity vs courier trust — mitigated by verifying originator sig on receiver; relay cannot forge.
- **Synthesis:** In-memory `RelayHub` + mpsc per session; persist discovery already exists; DHT later.

### Critic
- Acceptance testable on loopback (A→R←C)
- Anti-stub: no `todo!` / empty forward
- Verification: unit + integration + clippy `-D warnings`
- **Verdict: APPROVE**

### Acceptance
- [x] Address book optional `via`
- [x] `peer.relay.deliver` make/parse + hub register/deliver
- [x] `send_relayed_envelope` + recv allow-relayed for any signed type
- [x] CLI: `peer add --via`, listen `--relay`, `peer relay-hold`, send/trust-send honor `via`
- [x] Integration test A delivers to C via R without A dialing C
- [x] docs/peer-link.md + QUEUE update
- [x] CODE_REVIEW APPROVE/CLEAR

### Delivered
- `RelayHub` / `serve_relay_peer` / `send_envelope_to_peer`
- `PeerEndpoint.via` + CLI wiring
- ADR follow-through from Analyze-43

### Out
STUN, ICE, DHT, persistent disk session store, public bind default, multi-hop relay mesh.
