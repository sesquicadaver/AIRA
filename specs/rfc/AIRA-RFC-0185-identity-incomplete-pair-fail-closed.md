# AIRA-RFC-0185 — Identity incomplete-pair fail-closed (RFC-D)

## 1. Summary

Phase S `#298`: Desktop `ensure_local_identity` mints a new install identity **only** when both `local.identity.json` and `local.ed25519` are absent. A partial pair is fail-closed corruption (no silent new ID). Secret creation uses `create_new` and Unix `0600` without ignoring chmod errors. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

After `#276`, unique IDs were allocated on mint, but bootstrap treated “missing either file” as mint. Losing one identity file on an existing install caused a new key + new ID on next start, breaking provenance/trust linkage. Secret `chmod` failures were ignored after write.

## 3. Motivation

`phase-s-plan` `#298` / post-R audit §4: incomplete or invalid identity pair must not auto-mint.

## 4. Scope

- Partial pair → `#298` error; no overwrite of the remaining file
- Empty pair → mint with `create_new` + fail-closed `0600`
- Tests: orphan json, orphan secret, minted perms
- QUEUE → `#299`

## 5. Non-Goals

```text
Help connect boundary (#299)
Opt-in peer dial (#300)
CLI `aira identity create` rewrite
Silent recovery / re-pair UX wizard
```

## 6. Compatibility / Security

Complete existing installs unchanged. Corruption surfaces as bootstrap error instead of identity replacement.

## 7. Rollout

QUEUE `#298` → Analyze-334 → PR; next `#299` Connect Help boundary honesty.
