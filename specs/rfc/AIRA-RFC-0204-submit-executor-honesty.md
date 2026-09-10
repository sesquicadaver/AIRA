# AIRA-RFC-0204 — Submit executor honesty (RFC-D)

## 1. Summary

Phase U `#319`: staff CLI/HTTP/Desktop submit path binds execution-llm via `AIRA_LLM_BACKEND` (`with_backend_from_env`) and always labels reference mock distinctly from activate-ready. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

`LocalSession` rebound activate with forced `MockBackend`, so ProcessBackend existed in-library but staff path never used it. Activate strip "ready"/"готово" read as if a real model would run.

## 3. Motivation

`phase-u-plan` `#319` / post-T audit §7: staff submit must be configured process **or** explicit reference/mock mode.

## 4. Scope

- `OperationalPlane::bind_activate_gate` → `with_backend_from_env`
- `enable_activated_mock_llm` still forces mock for tests
- `staff_executor_kind` + CLI `executor … (reference)` / `mode reference`
- Desktop strip/System: ready · reference mock; executor field
- Tip → `#320`; RFC-D **0204**

## 5. Non-Goals

```text
AddressBook selective rollback (#320)
Changing mock payload semantics
GPU marketplace / remote LLM
Merging with other U atoms
```

## 6. Compatibility / Security

CI default remains mock. Process opt-in via env unchanged from RFC-0110 semantics.

## 7. Rollout

QUEUE `#319` → Analyze-355 → PR; next `#320` AddressBook selective rollback.
