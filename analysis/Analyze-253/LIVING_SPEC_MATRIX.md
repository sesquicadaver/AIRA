# Living spec — Analyze-253 (QUEUE #218)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Forged pointer fail-closed | `ActivatedPointerGate` | `forged_model_ref_only_pointer_is_denied`; `forged_model_ref_pointer_is_capsule_failed` | **DONE** |
| Hash of cache bytes | `check_activated` | `cache_hash_mismatch_is_denied` | **DONE** |
| Evidence + signature | `verify_activate_evidence` | `fixture_pointer_allows_generate`; `phase_d_activated_pointer_allows_mock_generate` | **DONE** |
| HTTP generate with real fixture | `install_fixture` | `http_post_problem_generate_with_activate_is_executed_not_verified` | **DONE** |
| C1 2+2 unchanged | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
| RFC-0111 reserved | no file | `phase_l_rfc_0111_id_free` | **DONE** |
