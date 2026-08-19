# Analyze-55 — mTLS CN → TrustStore

**QUEUE:** #20 DONE  
**Decision:** **A** (CN = full AiraRef)  
**Status:** CLOSED (APPROVE/CLEAR + UltraQA PASS)

## Delivered
- `TrustMappedClientVerifier`: WebPki CA then CN ∈ TrustStore ∧ ¬revoked
- Tests: trusted / unknown / revoked / wrong-CA / anon
- docs/local-node + QUEUE + crypto Out

## Out
SAN mapping; short CN; optional client auth; separate health (#21).
