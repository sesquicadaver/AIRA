# AIRA-RFC-0149 — Desktop action/error/help ID lexicon (RFC-D)

## 1. Summary

Phase O `#258`: introduce stable `ActionId` / `ErrorCode` / `HelpId` keys and `UiProblem` (`code → localized message → help_id`). GUI errors bind to the Phase O help catalog without implementing the F1 shell. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

Desktop surfaced raw `anyhow` strings with no stable codes or Help binding. F1 (`#263`) needs durable IDs that survive label renames.

## 3. Motivation

UX canon: shared lexicon between UI and Help; renaming a button must not break F1.

## 4. Scope

- `crates/aira-desktop/src/lexicon.rs` — catalog + gates + `UiProblem`
- Wire `last_problem` through Work/Node/status paths (and Generic for invite/discovery)
- Display message + stable `code` / `help:` keys; technical detail collapsed
- Unit tests; RFC-D this file; QUEUE → `#259`

## 5. Non-Goals

```text
Shell IA rewrite (#259)
Offline F1 renderer (#263)
Seed help Markdown articles (#264)
Policy enforcement changes in Core
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

No Core/ledger changes. Error details may still include technical strings; secrets must not be added to `detail`.

## 7. Rollout

QUEUE `#258` → Analyze-293 → PR; next `#259` Shell IA.
