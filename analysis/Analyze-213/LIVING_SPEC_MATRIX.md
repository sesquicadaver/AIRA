# Living spec — Analyze-213 (QUEUE #178)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Schema Pack §24 Promotion Candidate | `schemas/research/promotion-candidate.schema.json` | `promotion_candidate_schema_loads` | **DONE** |
| Required fields | fixtures valid + missing `source_artifact_ref` / unsigned | `schema validate --fixtures` | **DONE** |
| Runtime reject as operational | — | — | **OUT** (`#179`) |
