# Living Spec Matrix — Analyze-93

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC-D Inventory CSU | `specs/rfc/AIRA-RFC-0006-model-inventory-csu.md` + `csu/model-inventory` | manifest scoped/no-network |
| RFC-E CLI | `specs/rfc/AIRA-RFC-0007-models-scan-list-cli.md` + `commands/models.rs` | scan/list smoke |
| Scoped FS | `ensure_within_scope` | outside path fails |
| Immutable inventory | `CasArtifactStore::publish` CustomArtifact | roundtrip test |
| No C1 change | `aira-flow` plane | C1 conformance still green |
| No download | scan API | `downloadable_compatible_models=[]` |
