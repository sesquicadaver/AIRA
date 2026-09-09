# Analyze-345 — Observe miss fail durable

**QUEUE:** `#309` · **RFC-D:** [AIRA-RFC-0195](../../specs/rfc/AIRA-RFC-0195-observe-miss-fail-durable.md)

## Outcome

`activated.observe-fail.json` sticks mismatch/evidence fail to the current pointer/cache version. UI observe returns that fail without another streaming hash until the version changes.
