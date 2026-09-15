| ТЗ / atom | Модуль | Тести |
|-----------|--------|-------|
| Auto/specific preference | `work_readiness::WorkExecutorPreference` + Work UI | `work_readiness` unit; desktop compile |
| Math ≠ generate readiness | `evaluate_work_readiness` + `problem_binds_math_eval_safe` | math_is_ready_*; generate_* |
| Pre-submit admission | `submit_*_with_admission` + `AdmissionConstraints` | async_jobs + work gate |
| Choice ≠ VERIFIED | UI/i18n + reasons | copy in RFC-0232 acceptance |
| Не mock/result | — | anti-merge `#350` |
| Не Compare | — | anti-merge `#354` |
