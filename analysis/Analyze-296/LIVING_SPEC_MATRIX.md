# Analyze-296 — Living Spec Matrix

| ТЗ / канон | Модуль | Тести |
|------------|--------|-------|
| `desktop-ux` §4 чотири секції | `ui.rs` `ui_system*` | i18n section labels |
| UNKNOWN ≠ OFFLINE | `system_view::ConnectionConclusion` | `unknown_top_level_is_not_offline` |
| AddressBook ≠ live sessions | `SystemStatusView` | `address_book_not_confused_with_unobserved_sessions` |
| Model not invented | `ModelConclusion::NotChecked` | unit + copy |
| RFC-D | `AIRA-RFC-0152` | `phase_o_doc` |
| QUEUE | `#261` DONE → `#262` | `phase_o_queue_261_done_262_open` |
