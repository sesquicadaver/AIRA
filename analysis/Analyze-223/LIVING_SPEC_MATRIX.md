# Living spec — Analyze-223 (QUEUE #188)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Bound gate in invoke | `aira-csu` `CsuRuntime::invoke` take/restore | `invoke_binds_policy_gate_check_policy_allows` | **DONE** |
| Unknown effect ≠ Isolation | bound gate, unlisted action | `invoke_bound_gate_unknown_effect_is_deny_not_isolation` | **DONE** |
| Fail-closed without gate | `check_policy` | `check_policy_fail_closed_without_bound_gate` | **DONE** |
| RFC | `AIRA-RFC-0086-csu-policy-gate-invoke.md` | `phase_i_policy_gate_invoke_188` | **DONE** |
| Durable reuse | LocalSession | — | **OUT** (`#189`) |
