# AIRA-RFC-0063 — Policy Gate dispatch enforcement

## 1. Summary

Phase F `#114`: `CsuRuntime::dispatch` / `dispatch_with_artifacts` evaluate `csu.dispatch` via bound `PolicyGate` before invoking handlers. Fail-closed when gate is unbound or decision is DENY/REQUIRE.

## 5. Non-Goals

New policy YAML; acquisition audit (#115).

## 15. Tests

`cargo test -p aira-csu`
`cargo test -p aira-conformance --lib`
`cargo run -p aira-cli -- conformance run --profile C0` (case `c0.csu.dispatch_policy`)
