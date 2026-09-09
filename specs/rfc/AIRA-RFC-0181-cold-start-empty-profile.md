# AIRA-RFC-0181 — Cold-start empty profile guidance (RFC-D)

## 1. Summary

Phase R `#293`: on System → Connection, a **P0** profile with an empty address book shows **one** guidance line plus the existing primary CTA. Copy must not claim Applied or CONNECTED. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

Cold-start / empty Developer Preview profiles already had CTA matrix (`#287`) and promoted controls (`#288`), but lacked a single “what’s available now / what to do next” line, so the screen still felt like unexplained tech status.

## 3. Motivation

`phase-r-plan` R7 / acceptance: cold-start UK Connection shows a next step without requiring Technical details; no fake Applied/CONNECTED.

## 4. Scope

- `cold_start::{is_cold_start_empty_profile, cold_start_forbids_connected_claim}`
- `Labels::conn_cold_start_guidance` EN+UK
- Connection UI renders guidance when P0 + empty book
- Living smoke tip `#293` DONE → `#294`; Analyze-329; RFC-0181

## 5. Non-Goals

```text
Consolidating RFC-0174 (#294)
Changing CTA priority matrix (#287)
Fake CONNECTED / Applied from settings alone
New Network tab
```

## 6. Compatibility / Security

UI + predicate only. SettingsApplyPhase honesty unchanged.

## 7. Rollout

QUEUE `#293` → Analyze-329 → PR; next `#294` RFC-0174 close.
