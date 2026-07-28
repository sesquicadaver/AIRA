# Provenance — Noise XX scope

**Cycle:** Analyze-35  
**Parent:** Analyze-32/33/34 peer stack; QUEUE #2

## Decision
Hello v1 (Ed25519 + signed X25519) → Noise_XX_25519_ChaChaPoly_BLAKE2s → encrypted frames via `snow`.

## Out of scope
IK pattern; NAT; gossip; trust-delta message types; mTLS.
