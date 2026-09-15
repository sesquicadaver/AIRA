# Living Spec — Analyze-384 / `#347`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| `--excluded-model-ref` | `cli.rs` / `problem.rs` | `parses_allowed_and_excluded_model_refs` |
| Freeze allowed−excluded | `constraints_from_submit_flags` | `allowed_minus_excluded_freezes_into_snapshot` |
| Empty after exclude reject | `problem.rs` | `empty_auto_within_set_after_exclude_rejects` |
| No `--temperature` | clap | `rejects_removed_temperature_flag` |
| No privacy/fallback flags | clap | `rejects_removed_privacy_and_fallback_flags` |
| RFC-0230 | `specs/rfc/AIRA-RFC-0230-…` | `phase_x_doc` |
| Не Settings GUI | — | anti-merge `#348` |
