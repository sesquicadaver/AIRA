# Analyze-297 — Living Spec Matrix

| ТЗ / канон | Модуль | Тести |
|------------|--------|-------|
| Saved ≠ Applied | `AppliedRuntimeSettings` | `profile_change_needs_restart_until_reapplied` |
| Immediate prefs ≠ restart | open_ui / autostart | `open_ui_change_does_not_affect_restart_relevant` |
| Settings groups + close≠stop | `ui_settings` + i18n | i18n assertions |
| RFC-D | `AIRA-RFC-0153` | `phase_o_doc` |
| QUEUE | `#262` DONE → `#263` | `phase_o_queue_262_done_263_open` |
