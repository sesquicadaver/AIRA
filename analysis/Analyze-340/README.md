# Analyze-340 — Reachability durability honesty

**QUEUE:** `#304` · **RFC-D:** [AIRA-RFC-0191](../../specs/rfc/AIRA-RFC-0191-reachability-durability-honesty.md)

## Outcome

Docs + tests state that replay persists before state save with no joint atomic commit. Challenge burn without DIRECT is an accepted crash window.
