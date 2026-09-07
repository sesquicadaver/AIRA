# AIRA-RFC-0155 — Seed Help topics (uk/en) + link check

## 1. Summary

Phase O `#264`: deepen offline Help articles for every `HelpId`, require four UX sections plus Related, validate `help:<id>` cross-links (EN/UK parity, no orphans), expose related chips in the F1 panel. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

`#263` shipped stub articles without depth or a fail-closed link checker.

## 3. Motivation

`desktop-ux` §7 seed catalog must be usable offline in uk/en with stable cross-links.

## 4. Scope

- Seeded `docs/help/{en,uk}/*.md` (≥500 chars, required sections, ≥1 related link)
- `help::links` longest-prefix `help:<id>` resolver + unit tests
- Related topic chips in Help panel
- `docs/help/README.md`; QUEUE → `#265`

## 5. Non-Goals

```text
Consolidating RFC-0146 body (#265)
Remote HTML / scripts / LLM-authored normative help
Full docs site link graph outside HelpId catalog
```

## 6. Compatibility / Security

No Core/ledger changes. Links are local catalog tokens only.

## 7. Rollout

QUEUE `#264` → Analyze-299 → PR; next `#265` RFC-0146 close.
