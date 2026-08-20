# Living Specification — Analyze-103 / QUEUE #68

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC local capability ad | `specs/rfc/AIRA-RFC-0017-local-capability-advertisement.md` | docs |
| ALLOW publish → capability CAS + pointer | `publish_local` | `publish_local_writes_signed_descriptors_from_cache` |
| scope_type=local | capability payload / pointer | same |
| DENY → no capability ad | same | `publish_local_deny_skips_capability_ad` |
| CLI publish\|share | `aira-cli` models | clap Share alias |
