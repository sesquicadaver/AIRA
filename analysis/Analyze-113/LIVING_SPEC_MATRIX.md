# Living Spec — Analyze-113 / #78

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Native UI | `crates/aira-desktop` (egui) | compile + clippy |
| Lifecycle via shared lib | `aira-desktop-runtime` | existing lifecycle tests |
| XDG autostart | `autostart.rs` | `tests/autostart.rs` |
| Settings persist | UI → `write_settings` + sync | manual + unit autostart |
| CLI gui | `aira desktop gui` | help |
| RFC | RFC-0027 | — |
