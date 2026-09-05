# Analyze-267 — Prime Port invariant (QUEUE #232)

## BRIEF

Implement `aira-peer::prime_port` (`P_AIRA` = 1491 primes in 49152–65535), fail-closed AIRA-owned peer/discv/relay binds and book/DHT endpoints, Desktop/CLI default `127.0.0.1:49157`, RFC-0124. Not `#233` preferred_port.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| `|P_AIRA|==1491` | `prime_port.rs` | `p_aira_cardinality_and_bounds` | **DONE** |
| Fail-closed non-prime bind | `listen`/`bind_udp`/book/DHT | `listen_rejects_non_prime_port`; upsert reject | **DONE** |
| Desktop default prime | `DEFAULT_PEER_LISTEN` | settings_p1/p2 | **DONE** |
| RFC-D | `AIRA-RFC-0124-prime-port.md` | `phase_n_rfc_0124_prime_port_present` | **DONE** |
| QUEUE advance | `#232` DONE; first OPEN `#233` | `phase_n_queue_232_done` | **DONE** |
| preferred_port | — | — | **OUT** (`#233`) |

## TODO_FIXME

None in-scope residual. `#233` = deterministic preferred selection.
