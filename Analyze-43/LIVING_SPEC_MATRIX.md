# Living Spec Matrix — Analyze-43

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Exact-envelope gossip A→B→C + dedupe | `gossip` + `session` relay | `gossip_trust_delta_a_to_b_to_c` |
| Relayed send/recv (issuer ≠ TCP peer) | `session` | same + IdentityMismatch fail-closed without allow |
| `gossip_seen.json` cap/dedupe | `GossipSeenLog` | `gossip_seen_dedupes_and_caps` |
| `discovery.json` persist/upsert | `PeerDiscoveryStore` | `discovery_record_persists_and_upserts` |
| CLI `--gossip` + `peer discovery` | `aira-cli` | clippy `-D warnings` |
| ADR relay-first / DHT later | provenance | `ADR-connectivity-relay-first.md` |
| No STUN/DHT/public bind | scope | CODE_REVIEW anti-stub |
| QUEUE #10 micro DONE | `QUEUE.md` | next = #10b relay-first NAT |
