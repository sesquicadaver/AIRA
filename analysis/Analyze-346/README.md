# Analyze-346 — Quit∥Submit completion

**QUEUE:** `#310` · **RFC-D:** [AIRA-RFC-0196](../../specs/rfc/AIRA-RFC-0196-quit-submit-completion.md)

## Outcome

Quit during submit arms `quit_after_stop` and defers Stop until submit settles, then Stop→Close. UI shows waiting copy; no rejected-Stop sticky flag.
