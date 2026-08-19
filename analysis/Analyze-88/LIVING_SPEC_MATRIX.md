# Living Spec Matrix — Analyze-88

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC-S payload only | `specs/rfc/AIRA-RFC-0001-model-artifact-payload-schema.md` | no Core/enum change |
| ModelArtifact payload | `schemas/model/artifact.schema.json` | `aira-schema` load + fixtures |
| Valid fixture | `fixtures/valid/model/artifact.json` | manifest valid pass |
| Invalid missing hash | `fixtures/invalid/model/artifact-missing-hash.json` | manifest invalid fail |
| No canonical type | `crates/aira-artifact` `ArtifactType` | not in diff |
