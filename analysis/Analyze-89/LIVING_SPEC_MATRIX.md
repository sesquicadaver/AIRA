# Living Spec Matrix — Analyze-89

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC-S profile payload only | `specs/rfc/AIRA-RFC-0002-model-profile-payload-schema.md` | no Core/enum change |
| ModelProfile payload | `schemas/model/profile.schema.json` | `aira-schema` load + fixtures |
| Valid fixture | `fixtures/valid/model/profile.json` | manifest valid pass |
| Invalid missing model_ref | `fixtures/invalid/model/profile-missing-model-ref.json` | manifest invalid fail |
| No canonical type | `crates/aira-artifact` `ArtifactType` | not in diff |
