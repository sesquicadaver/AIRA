# AIRA-RFC-0154 — Offline F1 Help shell (embed, search, context)

## 1. Summary

Phase O `#263`: embed `docs/help/{en,uk}/*.md` into `aira-desktop`, search over title/body, open Help·F1 from last problem or section default, render plain Markdown offline. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

Help·F1 chrome existed (`#259`) without local article bodies or search; users could not read topics without network/LLM.

## 3. Motivation

`desktop-ux` §7 requires F1 offline (no internet, model, or running node) with stable `help_id` routing.

## 4. Scope

- `crates/aira-desktop/src/help/`: `include_str!` catalog, `search_help_ids`, `render_markdown_plain`
- Stub articles for all `HelpId` catalog entries (deepen + link check → `#264`)
- Help panel: search, topic chips, offline note; context = `last_problem.help_id` → tab default
- Unit tests + living smoke; QUEUE → `#264`

## 5. Non-Goals

```text
Deep uk/en seed + link checker (#264)
Consolidating RFC-0146 body (#265)
Remote HTML / scripts / LLM-authored normative help
```

## 6. Compatibility / Security

No Core/ledger changes. Renderer strips Markdown chrome to plain text only — no remote fetch.

## 7. Rollout

QUEUE `#263` → Analyze-298 → PR; next `#264` Seed help topics.
