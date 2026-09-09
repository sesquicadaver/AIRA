# Analyze-344 — Opt-in dial off UI-thread

**QUEUE:** `#308` · **RFC-D:** [AIRA-RFC-0194](../../specs/rfc/AIRA-RFC-0194-opt-in-dial-off-ui-thread.md)

## Outcome

`try_spawn_dial` / `poll_dial` move opt-in dial off the egui thread. UI shows waiting copy; F1 and navigation stay available.
