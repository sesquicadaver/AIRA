# Living Spec Matrix — Analyze-34

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Unbounded TCP accept wait | `aira_peer::accept` | idle listen; handshake still timed |
| Multi hello-only dials | `session` + CLI daemon | `listen_accepts_multiple_hello_only_dials` |
| Dial smoke without recv | CLI `peer listen` (no `--recv`) | docs + test drop-after-hello |
| Optional envelope recv | CLI `--recv` / `--once --recv` | docs/peer-link.md |
| Loopback default | `listen` | existing reject non-loopback |
