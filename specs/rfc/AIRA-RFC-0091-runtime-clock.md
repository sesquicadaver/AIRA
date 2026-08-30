# AIRA-RFC-0091 — Runtime Clock

## 1. Summary

Phase I `#193`: operational `created_at` comes from a process [`Clock`](../../crates/aira-object/src/clock.rs). Runtime default is wall-clock UTC (`SystemClock`). Tests may install `FixedClock`. Protocol-crate fixture `mvp_timestamp()` stays the historical MVP string.

## 5. Non-Goals

Envelope freshness / replay (`#194`); instance-scoped clock (process `OnceLock` remains, same class as signer; `#196`); rewriting every unit-fixture `Timestamp::parse("2026-07-10T12:00:00Z")`.

## 10. Contract

```text
aira_object::now() → SystemClock unless set_clock(FixedClock)
aira_csu::support::mvp_timestamp() → now()  (name kept; not a fixed MVP stamp)
OperationalPlane ProblemStatement / make_event / make_artifact → now()
aira_protocol::mvp_timestamp() → still "2026-07-10T12:00:00Z"
```

## 15. Tests

```text
cargo test -p aira-object --lib clock
cargo test -p aira-flow --lib local_session_artifacts_are_not_all_mvp_fixed_timestamp
cargo test -p aira-flow --lib local_session_fixed_clock_stamps_artifacts
```
