# AIRA-RFC-0086 — CSU PolicyGate bound in invoke

## 1. Summary

Phase I `#188`: `CsuRuntime::invoke` passes the runtime's bound `PolicyGate` into `CsuExecutionContext`. Handler `check_policy` evaluates effect-level actions. Unbound gate remains fail-closed (`CsuError::Isolation`).

## 5. Non-Goals

Durable reuse index (`#189`); new policy YAML; changing `csu.dispatch` semantics (`#114`).

## 10. Contract

```text
invoke takes runtime PolicyGate into CsuExecutionContext for on_event, restores after
check_policy with bound gate → ALLOW | DENY | REQUIRE
check_policy without gate → Isolation ("policy gate not bound")
unknown effect action with bound gate → DENY (not Isolation)
```

## 15. Tests

```text
cargo test -p aira-csu --lib
```
