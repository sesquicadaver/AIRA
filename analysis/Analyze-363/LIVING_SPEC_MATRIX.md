# Living Spec Matrix — Analyze-363 / QUEUE `#326`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| reuse key ⊇ snapshot | `admission.rs` `reuse_catalog_key` | `reuse_catalog_key_*` |
| RequireNewExecution gate | `reuse.rs` / plane bind | `require_new_*` / session test |
| model-X ≠ text-only hit | `local.rs` record + bind | `same_text_different_model_ref_*` |
| default C1 reuse | reuse index + session | `local_session_repeat_problem_*` |
| RFC-0211 | specs/rfc | `phase_v_doc` |
