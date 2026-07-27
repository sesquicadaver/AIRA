# Analyze-32 — Peer Link P0 (no controlling center)

**Scope:** Framed TCP + mutual Ed25519 hello + one signed `ProtocolEnvelope`; admission via local `trust.json`; static address book.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. No controlling center — trust is local; introducer deferred
2. Reuse Book II `ProtocolEnvelope` + existing Ed25519 identity/trust
3. Authenticated link via mutual challenge (Noise XX deferred)
4. Loopback / explicit bind only in tests; no public HTTP
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** New crate `aira-peer` — length-prefixed TCP frames, Ed25519 hello, static `peers/address_book.json`
- **Why:** Ship authz+delivery in one cycle without X25519/Noise deps; envelope stays transport-agnostic
- **Reject:** DHT, libp2p, gossip, Noise XX, federation join, embedding net in `aira-protocol`
- **Follow-up:** Noise XX under same frame/envelope API

### Acceptance
- Two temp nodes: mutual trust + dial/accept hello → Accept
- Untrusted / revoked peer → reject before envelope
- One envelope send/recv; issuer must equal authenticated peer; sig valid
- Oversized/truncated frame → fail closed
- `cargo test -p aira-peer` + docs
