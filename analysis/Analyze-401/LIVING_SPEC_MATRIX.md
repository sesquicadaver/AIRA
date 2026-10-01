# Living Spec — Analyze-401 / `#393`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Short EN/UK Unknown label (≤ 40) | `i18n.rs` `sys_conn_unknown` | `phase_z_393_short_unknown`; desktop i18n |
| Label is not the Offline sentence | same | desktop i18n `assert_ne` |
| UNKNOWN ≠ OFFLINE remains in Help | `docs/help/*/network.connect.md` | `phase_z_393_short_unknown` |
| Boundary/cold chrome unchanged | `conn_*_guidance` | same test |
| Tip → `#394` | QUEUE / entry tips | `phase_z_queue_393_done_394_open` |
