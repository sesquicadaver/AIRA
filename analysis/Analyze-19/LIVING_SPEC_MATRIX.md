# Living Spec Matrix — Analyze-19

| ТЗ / Roadmap | Модуль | Тести |
|--------------|--------|-------|
| M11 POST /v1/problems | `aira-node` http | `http_post_problem_2_plus_2` |
| M11 GET /v1/problems/{id} | `aira-node` http | `http_problem_status_roundtrip` |
| M11 GET /v1/results/{id} | `aira-node` http | `http_get_result` |
| M11 GET /v1/artifacts/{id} | `aira-node` http | `http_get_artifact` |
| M11 GET /v1/events | `aira-node` http | `http_events_tail` |
| M11 GET /v1/capabilities | `aira-node` http | `http_capabilities` |
| M11 GET /v1/csu | `aira-node` http | `http_csu_list` |
| M11 POST /v1/csu/register | `aira-node` http | `http_csu_register` |
| M11 POST /v1/conformance/run | `aira-node` http | `http_conformance_c0` |
| M11 loopback default | clap `--listen` | docs + default `127.0.0.1:8787` |
| Immutability | verification scripts | `deny-originals.sh` |
