# Living Spec Matrix — Analyze-92

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC-S acquisition policy only | `specs/rfc/AIRA-RFC-0005-model-acquisition-policy-payload-schema.md` | no Core/enum change |
| AcquisitionPolicy payload | `schemas/model/acquisition-policy.schema.json` | `aira-schema` load + fixtures |
| Valid fixture auto_download=false | `fixtures/valid/model/acquisition-policy.json` | manifest valid pass |
| Invalid missing auto_download | `fixtures/invalid/model/acquisition-policy-missing-auto-download.json` | manifest invalid fail |
| No downloader / allowlist runtime | CLI / CSU | not in diff |
