# Living Spec Matrix — Analyze-361 / QUEUE `#324`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| AdmissionSnapshot type | `crates/aira-flow/src/admission.rs` | `default_for_text_*`, `serde_roundtrip_*` |
| Publish on admit | `plane.rs` | `submit_publishes_admission_snapshot_on_problem_submitted` |
| Context factor | `csu/context-basic` | `problem_submitted_with_admission_snapshot_in_context` |
| ProblemRecord persist | `local.rs` | `local_session_persists_admission_snapshot` |
| RFC-0209 | `specs/rfc/AIRA-RFC-0209-admission-snapshot.md` | `phase_v_doc` tips |
