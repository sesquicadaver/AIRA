# Living Spec — Analyze-381 / `#344`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| A/B verified slots independent | `lifecycle.rs` / `verify.rs` | `two_models_verified_and_available_independently_after_latest_moves` |
| A/B available after tip move | `activate.rs` slot write | same |
| Restart preserves both | `list_model_lifecycle` | same |
| latest ≠ sole auth | `activate_gate.rs` | `non_latest_available_model_admits_via_slot_not_latest_tip` |
| RFC-0227 | `specs/rfc/AIRA-RFC-0227-…` | `phase_x_doc` |
| Не select API / GUI | — | anti-merge `#345`/`#348` |
