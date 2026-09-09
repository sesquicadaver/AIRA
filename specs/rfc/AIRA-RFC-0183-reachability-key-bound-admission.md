# AIRA-RFC-0183 — Reachability key-bound admission (RFC-D)

## 1. Summary

Phase S `#296`: `apply_successful_probe` rejects imported reachability evidence when `target_identity_ref` matches the current root identity but `target_public_key` is not the authoritative root-scoped local verifying key. Self-consistent same-ID / foreign-key packages no longer set DIRECT. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

Phase Q `#281` bound apply to the root **identity name** and verified signatures against the **embedded** `target_public_key`. An impostor root sharing the same identity string could produce a package that verified under its own key and was accepted onto a victim install.

## 3. Motivation

`phase-s-plan` `#296` / post-R audit: admission must be key-bound to the install keyring, not only name-bound.

## 4. Scope

- Root keyring gate on `target_public_key` before crypto/replay side effects
- Negative test: same identity name, foreign key → reject; state + replay unchanged
- QUEUE → `#297`

## 5. Non-Goals

```text
Model descriptor reopen (#297)
Identity incomplete-pair (#298)
Opt-in peer dial (#300)
```

## 6. Compatibility / Security

Legitimate packages (challenge signed for the current root key) still apply. Foreign-key packages with matching identity strings are rejected with a `#296` reachability error before DIRECT mutation.

## 7. Rollout

QUEUE `#296` → Analyze-332 → PR; next `#297` Model descriptor root-scoped verify.
