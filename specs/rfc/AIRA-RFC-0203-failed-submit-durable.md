# AIRA-RFC-0203 — Failed submit durable (RFC-D)

## 1. Summary

Phase U `#318`: after ProblemSubmitted accept, `LocalSession` always persists plane events and a `failed` problem row even when the pipeline returns `Err` (no VRA / CapsuleFailed). Reopen can read problem status and CapsuleFailed / FailureEvidenceCreated from the durable log. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

`submit_problem` used `plane.submit_problem(text)?` so any post-accept `Err` skipped `persist_after_submit`. Failure events existed only in memory and vanished on reopen.

## 3. Motivation

`phase-u-plan` `#318` / post-T audit §6: operators must retain failure history for accepted problems.

## 4. Scope

- `persist_failed_submit` + shared `persist_plane_events`
- `ProblemRecord.status = "failed"`
- Test: generate CapsuleFailed via LocalSession → reopen → problem + FailureEvidence
- QUEUE tip → `#319`
- RFC-D **0203**

## 5. Non-Goals

```text
Submit executor honesty / mock labeling (#319)
Changing plane Err → Ok(Failed) public outcome enum
Fixing reduction defaulting `/` expressions to `2+2`
Merging with other U atoms
```

## 6. Compatibility / Security

Success paths unchanged. Early Err before ProblemSubmitted still does not persist.

## 7. Rollout

QUEUE `#318` → Analyze-354 → PR; next `#319` submit executor honesty.
