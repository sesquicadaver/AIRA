# Provenance — trust-delta scope

Wire: `peer.trust.delta` / `aira:peer:trust-delta:v1`.
Ops: revoke, rotate, unrevoke via TrustStore.
CLI: `peer trust-send`; listen `--apply-trust` (requires `--recv`).
Out: auto-notify UX, gossip, DHT, coordinated Ed25519+X25519 rotate.
