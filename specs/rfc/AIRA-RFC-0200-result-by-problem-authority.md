# AIRA-RFC-0200 — Result-by-problem ArtifactStore authority (RFC-D)

## 1. Summary

Phase U `#315`: `LocalSession::get_result(problem_id)` treats `problems/index.json` as a **locator** only (`verified_artifact_id` / `execution_artifact_id`). The result body is always loaded through ArtifactStore `resolve` (descriptor + content-hash verify). Cached `ProblemRecord.result` is never authoritative alone. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

`get_result` returned `ProblemRecord.result` from the mutable problems index before any ArtifactStore check. CLI `result get` by problem id could therefore surface a tampered cache as if it were a verified result.

## 3. Motivation

`phase-u-plan` `#315` / post-T audit §3: problem-id lookup must not skip integrity checks that artifact-id lookup already performs.

## 4. Scope

- `get_result`: problem id → artifact ref → `get_artifact` / store resolve
- Fail-closed when no artifact locator remains (orphan cached `result` rejected)
- Tests: tampered index cache ignored; orphan result rejected
- QUEUE tip → `#316`
- RFC-D **0200**

## 5. Non-Goals

```text
CLI identity create fail-closed (#316)
Policy audit uniqueness (#317)
Stopping persistence of optional display cache in ProblemRecord.result
Merging with other U1 atoms
```

## 6. Compatibility / Security

`problem_status` may still expose the index `result` field for UI/status. Authority for `get_result` / CLI result path is ArtifactStore only.

## 7. Rollout

QUEUE `#315` → Analyze-351 → PR; next `#316` CLI identity create fail-closed.
