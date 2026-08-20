# Living Spec Matrix — Analyze-90

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC-S inventory payload only | `specs/rfc/AIRA-RFC-0003-model-inventory-payload-schema.md` | no Core/enum change |
| LocalModelInventory payload | `schemas/model/inventory.schema.json` | `aira-schema` load + fixtures |
| Valid fixture | `fixtures/valid/model/inventory.json` | manifest valid pass |
| Invalid missing signature | `fixtures/invalid/model/inventory-missing-signature.json` | manifest invalid fail |
| No canonical type | `crates/aira-artifact` `ArtifactType` | not in diff |
| No CLI/CSU | `crates/aira-cli` / inventory CSU | not in diff |
