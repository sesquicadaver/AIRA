# Analyze-293 — Living Spec Matrix

| ТЗ / atom | Модуль / артефакт | Тест | Статус |
|-----------|-------------------|------|--------|
| Help catalog IDs | `HelpId::catalog` | `help_catalog_matches_phase_o_seed_ids` | **DONE** |
| Action/error uniqueness | `ActionId` / `ErrorCode` | `action_and_error_ids_are_unique` | **DONE** |
| code→message→help | `UiProblem` | `problem_binds_code_message_help` | **DONE** |
| Submit classify | `from_submit_err` | `empty_submit_classifies_to_work_empty_text` | **DONE** |
| Action gate | `work_submit_gate` | `work_submit_gate_blocks_when_inflight` | **DONE** |
| RFC-D | `AIRA-RFC-0149-…` | `phase_o_rfc_0149_present` | **DONE** |
