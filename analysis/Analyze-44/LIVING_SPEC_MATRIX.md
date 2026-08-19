# Living Spec Matrix — Analyze-44

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Address book `via` | `address_book` | integration + CLI list |
| `peer.relay.deliver` make/parse | `relay` | shape unit + integration |
| Live hub register/deliver | `RelayHub` / `serve_relay_peer` | `hub_register_deliver_unregister`, `relay_hub_delivers_trust_delta_a_to_c_via_r` |
| Courier send honors `via` | `send_envelope_to_peer` | same integration |
| Relayed recv any signed type | `session` | hold path in integration |
| CLI `--relay` / `relay-hold` / `--via` | `aira-cli` | clippy build |
| No STUN/DHT | scope | CODE_REVIEW |
| QUEUE #10b DONE | `QUEUE.md` | next = #11 or DHT micro |
