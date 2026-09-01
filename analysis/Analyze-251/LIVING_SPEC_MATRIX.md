# Living spec — Analyze-251 (QUEUE #216)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Work generate via existing POST /v1/problems | `submit_problem` → `submit_desktop_problem` → node HTTP | `posts_generate_local_and_parses_executed_not_verified`; `http_post_problem_generate_with_activate_is_executed_not_verified`; `phase_k_desktop_generate_216` | **DONE** |
| Human-first UX (not raw VRA) | `format_work_result`; Work tab labels | `executed_generate_local_leads_with_result_not_verified`; `completed_vra_leads_with_answer_and_verified_not_hashes` | **DONE** |
| C1 2+2 stays math.eval.safe | execution-basic | `calculate_two_plus_two_stays_execution_basic`; `c1.pipeline.calculate_2_plus_2` | **DONE** |
| Activate gate fail-closed on Desktop path | LocalSession `ActivatedPointerGate` | `http_post_problem_generate_without_activate_is_not_verified`; `generate_without_activate_is_capsule_failed` | **DONE** |
| MockBackend remains CI default | OperationalPlane `with_mock_backend` | `default_plane_keeps_mock_backend` | **DONE** |
| RFC-0104 consolidates K | `AIRA-RFC-0104-phase-k-local-llm-csu.md` | `phase_k_rfc_0104_present`; `phase_k_desktop_generate_216` | **DONE** |
| RFC-0104 does not add anti-mission | Non-Goals GPU marketplace / LLM-in-Core / blockchain | `phase_k_rfc_0104_present` | **DONE** |
| QUEUE K closed | `#209`–`#216` DONE; no OPEN K | `phase_k_queue_wiring_209_done` | **DONE** |
