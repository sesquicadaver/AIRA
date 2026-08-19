# Правила Analyze-32

Peer link P0.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. No DHT / libp2p / gossip / Noise in this slice
3. Admission only via local `trust.json` (+ CRL)
4. Envelope issuer must bind to authenticated peer identity
5. Document in `docs/peer-link.md`
