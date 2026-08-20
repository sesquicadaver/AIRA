# Living Spec Matrix — Analyze-91

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC-S compatibility evidence only | `specs/rfc/AIRA-RFC-0004-model-compatibility-evidence-payload-schema.md` | no Core/enum change |
| CompatibilityEvidence payload | `schemas/model/compatibility-evidence.schema.json` | `aira-schema` load + fixtures |
| Valid fixture | `fixtures/valid/model/compatibility-evidence.json` | manifest valid pass |
| Invalid missing reason | `fixtures/invalid/model/compatibility-evidence-missing-reason.json` | manifest invalid fail |
| No rating score | schema properties | no rating_* fields |
| No canonical type / resolver | `aira-artifact` / CLI | not in diff |
