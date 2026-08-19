# Living Spec Matrix — Analyze-67

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Signed UDP announce | `aira_peer::discv` | `udp_announce_roundtrip_stores_dht_not_book` |
| Trust admission | `apply_discv_announce` | `untrusted_udp_announce_rejected`, `revoked_udp_announce_rejected` |
| identity_id == key_ref | `apply_discv_announce` | `identity_mismatch_on_key_ref` |
| Loopback bind default | `bind_udp` / `bind_udp_explicit` | `bind_udp_rejects_non_loopback_without_explicit` |
| Same DHT table, no book | `PeerDhtStore` source=`udp` | roundtrip asserts empty address book |
| CLI listen/announce | `aira-cli` PeerDiscv | compile + library helpers |
