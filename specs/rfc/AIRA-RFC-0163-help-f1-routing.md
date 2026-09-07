# AIRA-RFC-0163 — Help F1 routing (RFC-D)

## 1. Summary

Phase P `#273`: contextual Help·F1 clears stale `help_search`, routes **focus → section → screen** (never stale `last_problem`), shows localized article titles in topic/related lists, and does not auto-replace the pinned topic with `hits[0]` when search excludes it. RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

`open_help` changed the topic but left `help_search`; the panel forced `help_topic = hits[0]` when the topic was filtered out. F1 used `last_problem` / tab defaults instead of active element. Topic chips showed raw `HelpId` strings.

## 3. Motivation

Post-O audit §6 / `phase-p-plan` `#273`: old search must not override F1; routing must match documented focus→section→screen; titles must be user-facing.

## 4. Scope

- `help_focus`, `resolve_help_routing`, `set_tab` clears focus
- `open_help` clears search; panel keeps pinned topic outside hits
- Localized titles in topics/related; i18n `help_search_miss`
- Unit tests; QUEUE → `#274`

## 5. Non-Goals

```text
Consolidating RFC-0156 (#274)
New Help articles
LLM-authored Help
```

## 6. Compatibility / Security

Offline embedded Markdown only; no network/LLM. Explicit problem Help button still calls `open_help(problem.help_id)`.

## 7. Rollout

QUEUE `#273` → Analyze-308 → PR; next `#274` consolidating RFC-0156.
