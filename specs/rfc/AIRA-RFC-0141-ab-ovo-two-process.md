# AIRA-RFC-0141 — Real ab ovo two-process harness (RFC-D)

## 1. Summary

Phase N-fix `#249`: CI harness runs **two OS processes** with independent `--root` directories, a shared `LocalFileRendezvousProvider` ledger, empty AddressBook on B, then discover → explicit trust → dial Noise. In-process `#245` remains; this atom proves process isolation. RFC-0139 stays file-free until `#254`.

## 2. Problem Statement

`#245` ab ovo integration is same-process. Post-N audit requires an honest two-process harness before claiming real ab ovo.

## 3. Motivation

Cross-process ledger visibility and dial must not rely on shared in-memory provider state.

## 4. Scope

- Binary `aira-ab-ovo-harness` (`publisher` / `discoverer` roles)
- Integration test `ab_ovo_two_process`
- RFC-D this file; QUEUE → `#250`

## 5. Non-Goals

```text
Reachability session bind (#250)
Live Amoy HTTPS
Auto-trust from ledger
RFC-0139 consolidating body (#254)
```

## 6. Compatibility / Security

Additive. `DISCOVERED ≠ TRUSTED`. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#249` → Analyze-284 → PR; next `#250` reachability session bind.
