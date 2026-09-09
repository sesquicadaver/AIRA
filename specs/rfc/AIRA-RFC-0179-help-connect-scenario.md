# AIRA-RFC-0179 — Help connect scenario (RFC-D)

## 1. Summary

Phase R `#291`: Help articles `network.connect` (EN+UK), with linked updates to `network.trust` / `network.reachability`, describe the canonical scenario **invite → P1/P2 → Stop/Start → honest status** on the primary Connection surface — not only «understand the contract» under Technical details. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

Help for Connection led with honesty contracts and «open Technical details», so after Phase R promoted controls (`#288`) the docs still buried the job-to-be-done.

## 3. Motivation

`phase-r-plan` R5 / `desktop-ux`: Help must teach the same path the UI now surfaces as primary CTA.

## 4. Scope

- Rewrite `docs/help/{en,uk}/network.connect.md`
- Align `network.trust` / `network.reachability` «What to do» + Related with the scenario
- Living smoke tip `#291` DONE → `#292`; Analyze-327; RFC-0179
- Help seed/link tests remain green (≥500 chars; required sections; reciprocal Related)

## 5. Non-Goals

```text
UK mesh/discovery i18n parity (#292)
Cold-start empty-profile CTA (#293)
Consolidating RFC-0174 (#294)
New Network tab / Core changes
Public bind / auto-trust as default
```

## 6. Compatibility / Security

Docs-only for Help content; no weakening of UNKNOWN≠OFFLINE, book≠sessions, DISCOVERED≠TRUSTED.

## 7. Rollout

QUEUE `#291` → Analyze-327 → PR; next `#292` UK mesh/discovery parity.
