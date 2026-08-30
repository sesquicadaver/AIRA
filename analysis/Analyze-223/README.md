# Analyze-223 — CSU PolicyGate in invoke (QUEUE #188)

## Done
- `CsuRuntime::invoke` moves the bound `PolicyGate` into `CsuExecutionContext` for `on_event` and restores it
- `invoke_binds_policy_gate_check_policy_allows`; unknown effect is DENY not Isolation
- `check_policy_fail_closed_without_bound_gate`; RFC-0086
- QUEUE `#188` **DONE**; first OPEN `#189`

## Out
Durable reuse (`#189`); fail-closed signing (`#190`); new policy YAML.
