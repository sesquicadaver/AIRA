# Living Spec — Analyze-383 / `#346`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Auto-within-set freeze | `admission.rs` | `freeze_auto_within_set_picks_first_remaining_after_excludes` |
| Empty after excludes | `admission.rs` | `freeze_auto_within_set_empty_after_excludes_is_rejected` |
| Required∩excluded reject | `admission.rs` | `required_model_in_excluded_is_rejected` |
| Mid-flight ≠ admitted | `lib.rs` / `admission.rs` | `submit_with_admission_keeps_snapshot_after_settings_like_mutation` |
| Select honors excludes | `select.rs` | `auto_profile_excludes_tip_and_picks_remaining` |
| RFC-0229 | `specs/rfc/AIRA-RFC-0229-…` | `phase_x_doc` |
| Не CLI flags / GUI | — | anti-merge `#347`/`#348` |
