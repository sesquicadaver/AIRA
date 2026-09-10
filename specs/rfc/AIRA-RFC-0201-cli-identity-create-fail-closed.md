# AIRA-RFC-0201 — CLI identity create fail-closed (RFC-D)

## 1. Summary

Phase U `#316`: node identity mint is a **shared** fail-closed operation (`create_or_ensure_node_identity`). CLI `identity create` uses `CreateExclusive` — if key or descriptor already exists, return error and leave bytes unchanged. Desktop `ensure` uses `Ensure` — complete pair is a no-op. Descriptor signature is validated before any write. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

CLI `identity create` used `fs::write` and silently overwrote `local.ed25519` / `local.identity.json`. Desktop already refused overwrite via `create_new`, but the path was not shared.

## 3. Motivation

`phase-u-plan` `#316` / post-T audit §4: one create contract for CLI and Desktop; repeat create must not rotate secrets.

## 4. Scope

- `aira-object::create_or_ensure_node_identity` + `NodeIdentityCreatePolicy`
- CLI Create → `CreateExclusive`; Desktop ensure → `Ensure`
- Tests: second exclusive create fails unchanged; incomplete pair rejected; Desktop regressions
- QUEUE tip → `#317`
- RFC-D **0201**

## 5. Non-Goals

```text
Policy audit uniqueness (#317)
identity rotate / backup semantics
Changing Desktop display_name / unique desktop.<uuid> allocation
Merging with other U1 atoms
```

## 6. Compatibility / Security

Existing complete pairs are preserved. Incomplete pairs remain fail-closed (no mint).

## 7. Rollout

QUEUE `#316` → Analyze-352 → PR; next `#317` policy audit uniqueness.
