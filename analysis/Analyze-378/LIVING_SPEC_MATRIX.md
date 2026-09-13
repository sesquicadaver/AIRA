# Living Spec — Analyze-378 / `#341`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| swap VRA locator reject | `local::get_result` + `reuse::artifact_binds_problem_lookup` | `get_result_by_problem_rejects_swapped_verified_artifact_ref` |
| capsule↔event problem | `verification-basic` `capsule_matches_event_problem` | `swapped_capsule_output_refs_for_wrong_problem_are_not_verified` |
| generate ≠ false VF | generate early-return before expression | `generate_local_capsule_without_expression_is_not_verification_failed`; generate session tail |
| RFC-0225 | `specs/rfc/AIRA-RFC-0225-result-verify-task-binding.md` | `phase_w_doc.rs` |
