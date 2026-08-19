# Living Spec Matrix — Analyze-47

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Persist DHT + XOR closest | `PeerDhtStore` | `store_upsert_and_closest`, `xor_distance_*` |
| Announce signed + anti-spoof | `apply_dht_announce` | `announce_rejects_spoofed_identity` |
| A→B announce then find | integration | `dht_announce_a_to_b_then_find` |
| CLI dht + listen `--dht` | `aira-cli` | clippy build |
| No UDP/discv5/STUN | scope | CODE_REVIEW |
| QUEUE DHT micro DONE | QUEUE.md | next = mTLS / x25519 |
