# Analyze-79 — leftover test-sig fallback (QUEUE #44)

## Status
CLOSED (QUEUE #44 DONE @ 611c1b5 / PR #7).

## Done when
Runtime verify does not accept `LOCAL_TEST_DOMAIN_MSG` as a fallback after the intended message fails. Envelope verifies `payload_hash` only; identity create verifies `identity_id` bytes.

## Out
Re-rewriting Event/Artifact/Object/CSU canonical paths (#40–#43); new protocols; moving `LOCAL_TEST_DOMAIN_MSG` constant into `#[cfg(test)]` (still used for placeholder dummy signatures before canonical attach).
