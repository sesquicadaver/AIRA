# Living spec — Analyze-264 (QUEUE #229)

Матриця відповідності ТЗ → модуль → тести. Попередній атом: [Analyze-263](../Analyze-263/LIVING_SPEC_MATRIX.md).

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Honest constraints description | `schemas/execution/generate-local.schema.json` | `generate_local_payload_schema_loads` | **DONE** |
| OS layers not in payload | schema `constraints.description` | `generate_local_payload_schema_loads`; `phase_m_os_vs_aira_mediated_229` | **DONE** |
| RFC-0116 adapter none | schema `network.description`; RFC-0116 | `phase_l_network_none_222`; `phase_m_os_vs_aira_mediated_229` | **DONE** |
| Operator docs | `docs/local-node.md`; `docs/csu-development.md` | `phase_m_os_vs_aira_mediated_229` | **DONE** |
| RFC-D | `AIRA-RFC-0122-os-vs-aira-mediated.md` | `phase_m_os_vs_aira_mediated_229` | **DONE** |
| RFC-0117 reserved | no `AIRA-RFC-0117*` | `phase_m_rfc_0117_id_free` | **DONE** |
| QUEUE `#229` DONE | first OPEN `#230` | `phase_m_queue_wiring_224_done`; `phase_m_next_problem` | **DONE** |
| C1 2+2 | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
