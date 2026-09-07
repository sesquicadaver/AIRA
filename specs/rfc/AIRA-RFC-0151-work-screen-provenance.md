# AIRA-RFC-0151 — Desktop Work screen (user language, draft, Ctrl+Enter, provenance)

## 1. Summary

Phase O `#260`: rewrite the Work body for end-user language, keep the draft across chrome transitions, bind Ctrl+Enter / ⌘+Enter to Run, and surface honest provenance (mock ≠ verified; undefined model stated). RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

Work still spoke developer API (`POST /v1/problems`, CSU names) as primary copy and did not classify result origin for the user.

## 3. Motivation

`desktop-ux` §3 requires «Що потрібно зробити?» / «Виконати», newline vs Ctrl+Enter, draft retention, and explicit mock / missing-model provenance.

## 4. Scope

- i18n Work chrome (heading, Run, shortcut hint, user note + collapsed tech)
- Ctrl+Enter / ⌘+Enter submit; draft never cleared by submit path
- `ProvenanceKind` on `WorkResultView` + UI Origin line
- Human run-status labels for known wire statuses
- Tests; RFC-D this file; QUEUE → `#261`

## 5. Non-Goals

```text
System Program/Model/Events IA (#261)
Settings Saved≠Applied (#262)
Offline F1 articles (#263/#264)
Fake Cancel without executor support
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

No Core/ledger changes. Provenance is derived only from the response payload — never invents VERIFIED or a model name.

## 7. Rollout

QUEUE `#260` → Analyze-295 → PR; next `#261` System status screen.
