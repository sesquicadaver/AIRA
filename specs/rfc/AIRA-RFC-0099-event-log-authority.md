# AIRA-RFC-0099 — Event-log authority is file-chain

## 1. Summary

Phase J `#203`: after `LocalSession` persist, `event_tail` reads only `events/file-chain-log.json`. Reopen must not use `OperationalPlane` memory (`drain_from`) or fall back to legacy `events/event-log.json`. Dual-write to the legacy JSON file remains for `#142` / `#155` recovery helpers. Plane drain stays in-memory (C1 reference, not a production event runtime).

## 5. Non-Goals

Reduction catalog bind (`#204`); changing `drain_from` semantics or the 256 demo bound; RFC-0096 (`#208`); promoting OperationalPlane to production.

## 10. Contract

```text
event_tail — FileChainEventLog::open(events/file-chain-log.json)
missing/unreadable file-chain — fail-closed (no event-log.json fallback)
reopen plane.events() — must not be the source of persisted ProblemSubmitted
legacy event-log.json — recovery helper only
```

## 15. Tests

```text
cargo test -p aira-flow --lib event_tail_after_reopen_reads_file_chain_not_memory_or_legacy
cargo test -p aira-flow --lib session_durable_file_chain_roundtrip
cargo test -p aira-desktop-runtime --test phase_j_doc --test phase_i_doc --test phase_h_doc
```
