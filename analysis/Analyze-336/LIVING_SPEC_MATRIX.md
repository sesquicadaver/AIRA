# Living Spec Matrix — Analyze-336 / QUEUE `#300`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| explicit addr required | `peer_dial::run_opt_in_peer_dial` | `empty_addr_is_fail_closed` |
| trust required | same | `untrusted_peer_is_fail_closed` |
| dial + evidence file | same | `opt_in_dial_persists_evidence_without_direct_status` |
| no DIRECT from dial | mesh + reachability | same |
| live_session_count from fresh evidence | `network_mesh` | same |
| Desktop Technical dial UI | `ui` + `i18n` + `actions` | compile + i18n |
| RFC-0187 | `specs/rfc/AIRA-RFC-0187-…` | `phase_s_doc` |
| tip → first OPEN `#301` | QUEUE / plan / tips | `phase_s_queue_300_done_301_open` |
