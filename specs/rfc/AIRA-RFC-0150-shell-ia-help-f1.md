# AIRA-RFC-0150 — Desktop shell IA (Work / System / Settings + Help·F1)

## 1. Summary

Phase O `#259`: replace the four-tab E1 chrome (`Work / Node / Network / Settings`) with three primary sections (**Work / System / Settings**) plus a persistent **Help · F1** command. Esc closes Help only. Full offline articles remain `#263`/`#264`. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

End-user IA in `desktop-ux.md` targets three questions (do / status / settings) plus contextual Help. The running shell still exposed Node and Network as peer tabs to Work, which fights the Phase O language rule.

## 3. Motivation

Shell chrome must match the canon before Work copy (`#260`) and System honesty layout (`#261`) deepen each section.

## 4. Scope

- `MainTab::{Work, System, Settings}` + Help panel state (`help_open`, `help_topic`)
- Top chrome: three section tabs + persistent Help·F1; status strip (§2.2 placeholders)
- F1 opens contextual Help (`last_problem.help_id` → section default); Esc closes Help only
- Merge former Node + Network bodies under **System** (deeper Program/Model/Events → `#261`)
- i18n labels (`tab_system`, `help_f1`, strip strings); unit/i18n tests
- Docs + `phase_o_doc` advance; QUEUE → `#260`

## 5. Non-Goals

```text
Work body copy / Ctrl+Enter / provenance rewrite (#260)
System Program/Model/Events IA (#261)
Settings Saved≠Applied lifecycle (#262)
Offline Markdown F1 renderer (#263)
Seed help articles (#264)
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

No Core/ledger changes. Help panel is local chrome only; does not fetch network content.

## 7. Rollout

QUEUE `#259` → Analyze-294 → PR; next `#260` Work screen.
