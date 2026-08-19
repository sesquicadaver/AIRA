# LIVING_SPEC_MATRIX — Analyze-53

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Gossip skip subject≠issuer | `gossip_forward_trust_delta` | `gossip_skips_non_self_sovereign_trust_delta` |
| Self-sovereign still fans out | same | `gossip_trust_delta_a_to_b_to_c` |
| Seen mark on skip | `GossipSeenLog` | retry → duplicate skip in U1 |
| CLI skip reason | `aira-cli` peer listen --gossip | prints `gossip skipped (non-self-sovereign…)` |
| Docs | `docs/peer-link.md` | manual |
| QUEUE #18 | `QUEUE.md` | DONE |
