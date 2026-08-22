# Living Spec — Analyze-125 / #90

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Windows paths | `paths::windows_for_profile` | `windows_layout_under_profile` |
| Home derivation | `paths::windows_for_home` | `windows_for_home_derives_appdata_segments` |
| ensure_dirs | `DesktopPaths::ensure_dirs` | `windows_ensure_dirs_creates_tree` |
| RFC | RFC-0039 | — |
| Anti-stub | real path joins; no pass | CODE_REVIEW |
