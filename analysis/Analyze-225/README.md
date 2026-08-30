# Analyze-225 — Fail-closed signing (QUEUE #190)

## Done
- `active_signature` → `Result`; no `local_test_signature` fallback
- Default primary `aira:identity:local-test` remains explicit demo/test
- `LocalSession` bind_node_crypto propagates identity/trust/tenant errors
- `active_signature_does_not_fallback_to_local_test`; `local_session_rejects_corrupt_identity`; RFC-0088
- QUEUE `#190` **DONE**; first OPEN `#191`

## Out
Atomic persist (`#191`); artifact recovery (`#192`); instance-scoped crypto (`#196`).
