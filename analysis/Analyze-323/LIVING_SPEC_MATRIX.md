# Living Spec Matrix — Analyze-323 / QUEUE `#287`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| стан → рівно один primary CTA | `connection_cta.rs` | `primary_connection_cta` unit matrix |
| не лише Refresh на Unknown/LocalOnly | `app/ui.rs` `ui_sys_connection` | CTA module + i18n labels |
| RestartNeeded → Stop/Start | `connection_cta` + lifecycle | `restart_needed_running_is_stop` |
| P0 → enable private network | `connection_cta` + `apply_profile(P1)` | `p0_is_enable_private_network_not_refresh` |
| empty book → import invite | `connection_cta` + `import_json_dialog` | `empty_book_on_p1_is_import` |
| Direct/Relayed → no primary button | `ConnectionPrimaryCta::NoneOk` | `direct_with_peers_is_none_ok` |
| UNKNOWN≠OFFLINE | matrix never maps Unknown→Offline | `unknown_with_peers_is_refresh_not_offline` |
| RFC-D + tip `#288` | RFC-0175; QUEUE; `phase_r_doc` | living smoke |
| не `#288` promote | tech collapse unchanged | scope / Non-Goals |
