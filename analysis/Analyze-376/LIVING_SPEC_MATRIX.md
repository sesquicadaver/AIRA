# Living Spec — Analyze-376 / `#339`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| mock ≠ used-model | `csu/execution-llm` stamp + MockBackend | `mock_backend_completes_*`; `mock_claiming_used_model_*`; plane/e2e |
| mismatch reject | `ProcessBackend::with_expected_model_ref` | `process_expected_model_ref_mismatch_*` |
| binding into generate | `GenerateBackend::generate(..., &ExecutorFacts)` | process + mock unit |
| GUI defense | `work_view::extract_used_model` | mock used_model none |
| RFC-0223 | `specs/rfc/AIRA-RFC-0223-backend-verified-binding.md` | `phase_w_doc.rs` |
