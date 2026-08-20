# Living Spec Matrix — Analyze-94

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC resolver | `specs/rfc/AIRA-RFC-0008-model-compatibility-resolver.md` | docs |
| Classify rules | `csu/model-compatibility` `classify` | unit |
| Evidence publish | `resolve_and_publish` | roundtrip |
| CLI | `aira models compatible` | smoke |
| No CSU→CSU | Cargo.toml | dep_firewall |
| No download | API | no network code |
