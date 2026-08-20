# AIRA-RFC-0010 — Acquisition policy ALLOW path (RFC-D/E)

## 1. Summary

When a host-local acquisition policy exists and `auto_download=true`, `request_download` returns **ALLOW** and publishes a policy decision `CustomArtifact` (`decision: ALLOW`) plus `CustomEvent` (`op:policy-allowed:download:…`). ALLOW authorizes a later quarantine fetch (`#62`); it **MUST NOT** copy or fetch model bytes.

## 2. Problem Statement

D3 (`#60` / RFC-0009) proved default DENY. D4.1 must prove the complementary ALLOW decision so `#62` can gate on an explicit policy outcome without inventing informal JSON.

## 3. Scope

- Extend `GateDecision` with `Allow`
- `reason_ref`: `aira:reason:auto-download-true`
- CLI: `aira models download` exit **0** + `status policy-allowed` on ALLOW; DENY exit **2** unchanged
- Pointer `models/acquisition.decision.latest.json` records `decision: ALLOW`

## 4. Non-Goals

```text
quarantine fetch (#62)
hash/signature verify (#63)
activate (#64)
remote HTTP
sharing / rating
C1 / aira-core changes
```

## 5. Failure / decision table

| Condition | Decision | reason_ref | Exit |
|-----------|----------|------------|------|
| no policy | DENY | `aira:reason:no-acquisition-policy` | 2 |
| auto_download=false | DENY | `aira:reason:auto-download-false` | 2 |
| auto_download=true | ALLOW | `aira:reason:auto-download-true` | 0 |

## 6. Rollback

Revert ALLOW arm; restore RFC-0009 `download-not-implemented-d3` DENY for `auto_download=true`.

## 7. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6a D4.1; QUEUE `#61`.
