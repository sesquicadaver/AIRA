# Analyze-287 — netns NAT/firewall honesty (QUEUE #252)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Honest placeholder name | `configure_non_listening_placeholder_via_relay` | `configure_helper_is_non_listening_*` | **DONE** |
| Listening ≠ placeholder | `inbound_firewall` | `listening_socket_is_not_the_same_*` | **DONE** |
| Firewall DROP proof | `scripts/inbound_firewall_smoke.sh` | `inbound_firewall_docker_or_documented_path` | **DONE** |
| Relay path retained | `both_inbound_blocked_relay_courier_*` | same | **DONE** |
| RFC-D | `AIRA-RFC-0144-…` | `phase_n_fix_*` | **DONE** |
| Port wrap | — | — | **OUT** (`#253`) |
