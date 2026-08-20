# Living Specification — Analyze-99 / QUEUE #64

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC activate | `specs/rfc/AIRA-RFC-0013-activate-verified.md` | docs |
| verified → cache + Event | `activate_verified` | `activate_copies_verified_to_cache_no_execution` |
| missing verified | same | `activate_requires_verified_pointer` |
| inventory refresh | CLI `Activate` + `scan_and_publish(cache)` | smoke |
| executed=false | evidence + CLI | unit/smoke |
