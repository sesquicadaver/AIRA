# Analyze-230 — Run nonce concurrency (QUEUE #195)

## Done
- `alloc_run_nonce()` is UUIDv7 hex; no shared counter file
- CSU/plane `run_nonce` is a string namespace
- `alloc_run_nonce_concurrent_is_unique`; leftover `run-counter` ignored; RFC-0093
- QUEUE `#195` **DONE**; first OPEN `#196`

## Out
Instance-scoped crypto (`#196`); MSRV CI (`#197`).
