# Living Spec Matrix — Analyze-339 / QUEUE `#303`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| UI miss → pending, no caller hash | `activate_gate::observe` | `observe_ui_miss_defers_hash_without_caller_full_hash` |
| streaming hash API | `ContentHash::sha256_path` | activate_gate + admit tests |
| admit not weakened | `check_activated` | `admit_always_full_hashes_even_after_observe_cache` |
| Desktop uses UI observe | `ModelTripleSnapshot::load` | phase_s_doc + runtime |
| RFC-0190 | specs/rfc | `phase_s_rfc_0190_present` |
| tip → `#304` | QUEUE / tips | `phase_s_queue_303_done_304_open` |
