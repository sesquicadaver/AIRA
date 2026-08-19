# DI crystallize — Analyze-78 / QUEUE #43

## In
1. Manifest production signatures: canonical JSON without top-level `signature`, signed by `identity_ref`.
2. `validate_for_registration` uses `verify_canonical` only (no `csu_id` bytes).
3. Mutation tests for name, type, abi, publisher, csu_id.
4. `apply_publisher` re-signs so publisher is covered by the signature.

## Out
New CSU implementations; leftover envelope/identity LOCAL_TEST fallback (#44).
