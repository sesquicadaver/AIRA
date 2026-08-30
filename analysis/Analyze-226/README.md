# Analyze-226 — Atomic session persist (QUEUE #191)

## Done
- `write_json` writes `{name}.tmp` then `rename` onto the target
- `persist_after_submit` fails closed on a corrupt problems index (no `unwrap_or_default`)
- `local_session_corrupt_problems_index_is_not_silent_wipe`; RFC-0089
- QUEUE `#191` **DONE**; first OPEN `#192`

## Out
Artifact metadata recovery (`#192`); Clock (`#193`); multi-file transactional persist.
