# AIRA-RFC-0170 — Reachability evidence admission (RFC-D)

## 1. Summary

Phase Q `#281`: applying peer-assisted DIRECT evidence requires the challenge target to equal the current root identity, durable challenge-id replay under `peers/reachability_replay.json`, and apply-time freshness (challenge not expired at apply; `|applied_at − probed_at|` / evidence `created_at` within skew — not only the signed `probed_at` window). RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`apply_successful_probe` verified crypto/bindings but accepted any install root, skipped durable replay, and treated a historically valid signed `probed_at` as sufficient — stale or foreign evidence could set DIRECT.

## 3. Motivation

`phase-q-plan` `#281` / acceptance: evidence apply is root-bound, non-replayable, apply-time fresh.

## 4. Scope

- Root identity gate on apply
- Durable `ReachabilityReplayLog` + load/save
- `check_apply_time_freshness` + `REACHABILITY_APPLY_MAX_SKEW_SECS`
- CLI passes apply-time `now`
- QUEUE → `#282`

## 5. Non-Goals

```text
Lifecycle revision races (#282)
Transcript domain / schema bump beyond replay file
CGNAT netns CI
```

## 6. Compatibility / Security

Foreign-root apply rejected. Replayed challenge_id rejected across reopen. Apply beyond skew / after challenge expiry rejected even when signatures verify.

## 7. Rollout

QUEUE `#281` → Analyze-316 → PR; next `#282` Lifecycle revision races.
