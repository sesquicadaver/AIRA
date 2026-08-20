# AIRA-RFC-0013 — Activate verified model (RFC-D/E)

## 1. Summary

`activate_verified` copies the latest verified weight from `models/verified/` into `models/cache/`, publishes ModelInstalled-style Evidence (`activated=true`, `executed=false`) and Event `op:model-installed:activate:…`. CLI `aira models activate` then refreshes inventory by scanning `models/cache` only (CSU↛CSU firewall).

## 2. Problem Statement

Verify staging must not imply install. Activation is an explicit operator step and must not auto-execute models.

## 3. Scope

- `activate_verified(root)`
- Pointer `models/activated.latest.json`
- CLI `aira models activate` + inventory scan of cache
- Requires `#63` verified pointer

## 4. Non-Goals

```text
auto-execution / inference
sharing / rating / remote registry (D5–D7)
C1 / aira-core
acquisition → inventory Cargo dependency
```

## 5. Semantics

| Condition | Result | Exit |
|-----------|--------|------|
| no verified pointer | error `NoVerified` | non-zero |
| verified present | cache copy + Event + inventory refresh | 0 |

## 6. Rollback

Remove `activate_verified` and CLI `Activate`.

## 7. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6a D4.4; QUEUE `#64`.
