# Living Spec Matrix — Analyze-318 / QUEUE `#283`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| used ≠ backend id | `extract_used_model` | `used_model_never_takes_backend_id` |
| без model evidence → undefined/none | same + `used_model_fact` | mock/process/C1 cases in same test |
| model_ref wins over backend | `extract_used_model` | `with_ref` case |
| RFC-0172 | `specs/rfc/AIRA-RFC-0172-…` | `phase_q_rfc_0172_present` |
