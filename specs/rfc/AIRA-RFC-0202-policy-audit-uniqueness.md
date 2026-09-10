# AIRA-RFC-0202 — Policy audit uniqueness (RFC-D)

## 1. Summary

Phase U `#317`: each `PolicyEvaluated` event id embeds a run nonce (`aira:event:policy{nonce}_{seq}`). Persist treats same-id + same canonical content hash as idempotent and same-id ≠ hash as fail-closed conflict (no silent skip). Companion uniqueness for CSU lifecycle/failure ids uses the same run nonce so durable persist remains honest across submits. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

`PolicyGate` emitted `aira:event:policy{N}` with `N` resetting on every new gate. Each `submit_problem` rebuilt the plane/gate, so the second submit reused `policy1…`. `persist_after_submit` skipped by event id only, dropping later policy decisions from the durable log.

## 3. Motivation

`phase-u-plan` `#317` / post-T audit §5: policy decisions must remain distinguishable and durable; content-hash equivocation must not be ignored.

## 4. Scope

- `PolicyGate::with_run_nonce` / unique default nonce
- `CsuRuntime::set_run_nonce` + lifecycle/fail id binding
- `admit_persisted_event` on legacy + durable persist
- Tests: distinct policy ids across submits; same-id ≠ hash conflict
- QUEUE tip → `#318`
- RFC-D **0202**

## 5. Non-Goals

```text
Failed submit durable history (#318)
Trust-audit.jsonl uniqueness
Changing PolicyDecision payload hashing beyond event descriptor canonical hash
Merging with other U1 atoms
```

## 6. Compatibility / Security

Memory `EventLog` already rejected equivocation; durable path now matches. Existing single-submit logs remain readable.

## 7. Rollout

QUEUE `#317` → Analyze-353 → PR; next `#318` failed submit durable.
