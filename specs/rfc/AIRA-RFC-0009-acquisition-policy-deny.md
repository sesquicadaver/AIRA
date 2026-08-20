# AIRA-RFC-0009 — Acquisition policy default-DENY runtime (RFC-D/E)

## 1. Summary

CSU `aira:csu:model.acquisition` evaluates download requests under default-deny. Missing policy or `auto_download=false` → `DENY` + policy decision `CustomArtifact` + `CustomEvent` (`op:policy-denied:download:…`). No byte transfer (D4 Out).

## 2. Problem Statement

Without a gate, a future downloader could implicitly fetch models. D3 must prove DENY + Evidence before any acquisition CSU.

## 3. Scope

- Crate `csu/model-acquisition` (`network=none`)
- Optional policy file `models/acquisition.policy.json`
- `request_download` always refuses transfer in D3
- CLI: `aira models policy show|set`, `aira models download --model-ref …` (exit 2 on DENY)

## 4. Non-Goals

```text
real HTTP/CAS download (D4)
sharing
C1 plane change
allowlist marketplace
```

## 5. Failure Semantics

| Condition | Decision | reason_ref |
|-----------|----------|------------|
| no policy | DENY | `aira:reason:no-acquisition-policy` |
| auto_download=false | DENY | `aira:reason:auto-download-false` |
| auto_download=true | DENY (no transfer) | `aira:reason:download-not-implemented-d3` |

## 6. Rollback

Remove crate + CLI Policy/Download variants.

## 7. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) D3; QUEUE `#60`.
