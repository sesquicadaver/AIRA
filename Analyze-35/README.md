# Analyze-35 — Noise XX

**Scope:** Noise_XX_25519_ChaChaPoly_BLAKE2s after Ed25519 hello; encrypt envelope frames; bind Noise static to hello.

**Status:** DONE — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Keep Ed25519 trust admission as fail-closed gate
2. Encrypt all post-hello application frames (envelopes)
3. Bind Noise static X25519 to hello via signature
4. Same CLI surface; loopback default; no NAT/DHT/trust-delta
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### Decision Drivers
1. QUEUE #2 / A-32–33 follow-up
2. Proven `snow` crate
3. Minimal wire break: hello domain bump to v1

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Hello v1 + XX + encrypt frames** | Clear bind; identity first | Hello still cleartext |
| B. Pure Noise IK only | Fewer RTTs | Weaker trust.json reuse |
| C. NN after hello (no static) | Simple | No static bind |

**Chosen: A.**

### ADR
- **Decision:** `aira:peer:hello:v1` includes signed `x25519_pub_hex`; then Noise XX; transport encrypts length-prefixed frames. Node static at `identity/local.x25519` (auto-create). After XX, require `get_remote_static() == hello.x25519_pub`.
- **Reject:** IK-only; cleartext envelopes after hello; NAT
- **Follow-up:** trust-delta over encrypted link (QUEUE #3)

### Acceptance
- [x] Hello v1 requires signed X25519 pub
- [x] Dial/accept complete XX then encrypted envelope roundtrip
- [x] Mismatch remote static vs hello → fail closed
- [x] Multi dial + send still works with daemon listen
- [x] Tests + clippy; docs + QUEUE update
