# Analyze-221 — Handle integrity (QUEUE #186)

## Done
- `Handle::new` / `Handle::storage_token` are `pub(crate)`
- Store mint via `aira_object::object_store_access`
- `ObjectStore::open` bind → `CoreError::HandleBindMismatch`
- Adversarial tests (memory + sqlite); RFC-0084
- QUEUE `#186` **DONE**; first OPEN `#187`

## Out
Semantic verify (`#187`); PolicyGate invoke (`#188`); durable reuse (`#189`).
