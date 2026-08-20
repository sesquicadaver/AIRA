# AIRA-RFC-0015 — share_custom_models publish gate (RFC-D/E)

## 1. Summary

`request_publish` evaluates `share_custom_models` on the host acquisition policy. Absent policy or `false` → DENY + Evidence/Event (`op:policy-denied:publish:…`). `true` → ALLOW + Evidence (`op:policy-allowed:publish:…`) without writing ShareOffer bytes.

## 2. Problem Statement

D5.3 local publish must not run without an explicit opt-in gate, parallel to download DENY/ALLOW.

## 3. Scope

- `request_publish` in `csu/model-acquisition`
- Pointer `models/share.decision.latest.json`
- CLI: `aira models publish --model-ref`, `policy set --share-custom-models`
- `write_acquisition_policy(auto_download, share_custom_models)`

## 4. Non-Goals

```text
ShareOffer materialization (#67)
capability advertisement (#68)
remote registry / network share
rating (D6)
```

## 5. Decision table

| Condition | Decision | reason_ref | Exit |
|-----------|----------|------------|------|
| no policy | DENY | `no-acquisition-policy` | 2 |
| share_custom_models=false | DENY | `share-custom-models-false` | 2 |
| share_custom_models=true | ALLOW | `share-custom-models-true` | 0 |

## 6. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6b D5.2; QUEUE `#66`.
