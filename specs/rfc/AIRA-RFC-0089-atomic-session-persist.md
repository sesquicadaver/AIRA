# AIRA-RFC-0089 — Atomic session persist

## 1. Summary

Phase I `#191`: `LocalSession` metadata writes (`write_json`) use temp-file + rename. A corrupt `problems/index.json` fails closed; persist does not replace it with an empty index.

## 5. Non-Goals

Artifact metadata recovery (`#192`); transactional persist across events + problems + reuse as one commit; crash-inject tests of partial rename.

## 10. Contract

```text
write_json(path) → write path.tmp → rename over path
missing problems/index.json → empty index (first persist)
existing corrupt problems/index.json → Err; file bytes unchanged
```

## 15. Tests

```text
cargo test -p aira-flow --lib local_session_corrupt_problems_index_is_not_silent_wipe
cargo test -p aira-flow --lib local_init_submit_status_and_artifact
```
