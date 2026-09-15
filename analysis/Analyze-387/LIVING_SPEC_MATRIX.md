| ТЗ / atom | Модуль | Тести |
|-----------|--------|-------|
| Demo/mock banner + CTA | `ui_work` + `executor_is_reference_mock` | desktop compile; i18n labels |
| requested/applied/executed | `work_view::ResultModelTriple` + submit context | mock_result_triple_*; process_result_triple_* |
| Mock ≠ used-model | `EXECUTED_MOCK_LABEL` + extract_used_model | used_model_never_takes_backend_id |
| Не network UX | — | anti-merge `#351` |
| Не Compare | — | anti-merge `#354` |
