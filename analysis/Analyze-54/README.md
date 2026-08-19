# Analyze-54 — x25519 peer notify → WONT-NEED

**QUEUE:** #19 DONE  
**Decision:** **D** (hello-sufficient)  
**Status:** CLOSED (APPROVE/CLEAR; UltraQA skipped — docs-only)

## Rationale
Hello v1 already carries `x25519_pub_hex` signed by Ed25519 against TrustStore; Noise remote static is bound to that hello each dial. There is no durable remote Noise-static cache. A separate notify path would mirror Ed25519 rekey UX without improving admission.

## Delivered
- QUEUE #19 WONT-NEED; next OPEN #20
- docs/peer-link + crypto notes
- A-49 TODO closed
- Living Spec + review/QA artifacts

## Out
Notify / pin / dual-static grace (not implemented).
