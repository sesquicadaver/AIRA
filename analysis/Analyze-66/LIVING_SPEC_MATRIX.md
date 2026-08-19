# Living Spec Matrix — Analyze-66

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Binding + XOR-MAPPED | `aira_peer::stun` | `rfc5769_xor_mapped_ipv4_vector`, `mock_stun_roundtrip_and_persist` |
| Persist reflexive | `StunReflexiveRecord` | `mock_stun_roundtrip_and_persist` |
| Announce XOR flags | `resolve_dht_announce_addr` | `resolve_from_stun_and_conflict` |
| CLI query / --from-stun | `aira-cli` PeerStun / PeerDht | compile + helper tests |
| dial unchanged | `session::dial` | existing peer dial tests |
