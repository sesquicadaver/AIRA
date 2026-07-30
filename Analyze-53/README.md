# Analyze-53 — Gossip drops non-self-sovereign trust-delta

**Scope:** `gossip_forward_trust_delta` не форвардить envelope, якщо `subject_id ≠ issuer` (після parse). Mark-seen + skipped. Apply-політика A-52 без змін. No Manifesto/Meditation. No DHT/relay.

**Status:** CLOSED (APPROVE/CLEAR + UltraQA PASS)  
**QUEUE:** #18 DONE

## Acceptance
- [x] subject≠issuer → skip forward (no dial), seen marked
- [x] subject==issuer → forward as before (existing gossip test)
- [x] docs/peer-link + QUEUE #18 DONE
- [x] cargo test -p aira-peer + clippy
