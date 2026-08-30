# Living spec — Analyze-222 (QUEUE #187)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Independent eval | `csu/verification-basic` `math_eval_safe` | `verifies_math_output_as_verified_result` | **DONE** |
| Wrong finite | claimed 5.0 for `2+2` | `wrong_finite_math_result_is_not_verified` | **DONE** |
| Capsule expression | `artifact_refs[1]` | `math_expression_from_capsule_artifact` | **DONE** |
| RFC | `AIRA-RFC-0085-semantic-verify-math.md` | `phase_i_semantic_verify_187` | **DONE** |
| PolicyGate invoke | `CsuExecutionContext` | — | **OUT** (`#188`) |
