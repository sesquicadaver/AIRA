# Living Spec Matrix — Analyze-95

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC DENY gate | `specs/rfc/AIRA-RFC-0009-acquisition-policy-deny.md` | docs |
| No policy → DENY | `request_download` | unit |
| auto_download=false → DENY | same | unit |
| No transfer if true | same | unit (no weights) |
| CLI download exit 2 | `commands/models.rs` | smoke |
| No C1 change | plane | C1 conformance |
