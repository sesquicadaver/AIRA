# AIRA-RFC-0196 — Quit∥Submit completion (RFC-D)

## 1. Summary

Phase T `#310`: Quit while Work submit is in flight **defers** Stop until submit settles, then continues Stop→Close. The UI shows an explicit waiting label. `quit_after_stop` is never left as a sticky unused intent after a rejected Stop. RFC-0192 stays file-free until `#312`.

## 2. Problem Statement

After `#302`, submit⊥lifecycle rejected Stop during submit. `request_quit` still set `quit_after_stop = true` then called Stop (rejected). Submit completion did not continue Quit, so the flag stayed true and a later lifecycle could close unexpectedly.

## 3. Motivation

`phase-t-plan` `#310` / post-S audit §5: one contract — deferred Stop→Close **or** reject without sticky flag. Deferred with explicit message is the chosen UX.

## 4. Scope

- `quit_arm_policy` / `quit_followup_after_submit`
- `request_quit` defer path + `poll_submit` → QueueStop
- EN/UK `quit_waiting_submit` chrome
- QUEUE tip → `#311`

## 5. Non-Goals

```text
AddressBook candidate honesty (#311)
Changing submit⊥lifecycle exclusivity (#302)
Merging with dial atoms
```

## 6. Compatibility / Security

Lifecycle Quit chaining after Start (`#282`) unchanged. Submit still exclusive with Start/Stop.

## 7. Rollout

QUEUE `#310` → Analyze-346 → PR; next `#311` AddressBook candidate honesty.
