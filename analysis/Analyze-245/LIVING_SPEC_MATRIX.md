# Living spec — Analyze-245 (QUEUE #210)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Generate-local schema | `schemas/execution/generate-local.schema.json` `$id` `aira:schema:execution:generate-local:0.1` | `generate_local_payload_schema_loads`; `schema validate --fixtures` | **DONE** |
| Fixtures | valid generate-local + missing-prompt invalid + manifest | `fixture_manifest_passes` | **DONE** |
| RFC-S | `AIRA-RFC-0105-generate-local-payload-schema.md` | `phase_k_generate_local_210` | **DONE** |
| RFC-0104 reserved | no `AIRA-RFC-0104*` yet | `phase_k_rfc_0104_id_free` | **DONE** |
| QUEUE K | `#210` DONE, `#211`–`#216` OPEN | `phase_k_queue_wiring_209_done` | **DONE** |
| execution-llm CSU | `csu/execution-llm` | — | **OUT** (`#211`) |
