# Living spec — Analyze-240 (QUEUE #205)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Independent echo | `csu/verification-basic` `text_matches_claimed` | `verifies_text_echo_output_as_verified_result` | **DONE** |
| Independent uppercase | same | `verifies_text_uppercase_output_as_verified_result` | **DONE** |
| Wrong string echo | claimed `world` for `hello` | `wrong_text_echo_result_is_not_verified` | **DONE** |
| Wrong string uppercase | claimed `hello` for `hello` | `wrong_text_uppercase_result_is_not_verified` | **DONE** |
| Capsule expression | `artifact_refs[1]` | `text_echo_expression_from_capsule_artifact` | **DONE** |
| RFC-0101 | `specs/rfc/AIRA-RFC-0101-semantic-verify-text.md` | `phase_j_semantic_verify_text_205` | **DONE** |
| Evidence primacy | Claim vs Assumption runtime | — | **OUT** (`#206`) |
