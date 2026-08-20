# Living Specification — Analyze-104 / QUEUE #69

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC rating-evidence | `specs/rfc/AIRA-RFC-0018-model-rating-evidence-payload-schema.md` | docs |
| Schema `$id` | `schemas/model/rating-evidence.schema.json` | load + validate |
| Valid + context | `fixtures/valid/model/rating-evidence.json` | manifest valid |
| Invalid missing context | `fixtures/invalid/model/rating-evidence-missing-context.json` | manifest invalid |
| Unit | `aira-schema` | `model_rating_evidence_payload_schema_loads` |
