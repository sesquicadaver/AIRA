# Living Spec — Analyze-377 / `#340`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| foreign VRA ≠ reuse Complete | `reuse::admit_reuse_candidate` + `plane::bind_catalog_for_admission` | `foreign_vra_under_correct_reuse_key_does_not_complete_via_reuse`; `admit_rejects_foreign_math_result` |
| compatible reuse kept | same | `ready_solution_reuse_skips_execution`; `admit_accepts_compatible_math_result` |
| statement hash mismatch | `admit_reuse_candidate` | `admit_rejects_statement_hash_mismatch` |
| RFC-0224 | `specs/rfc/AIRA-RFC-0224-reuse-candidate-independent-check.md` | `phase_w_doc.rs` |
