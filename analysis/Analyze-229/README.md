# Analyze-229 — Envelope freshness/replay (QUEUE #194)

## Done
- `admit_envelope` policy window in `aira-protocol`
- `AuthenticatedPeer::recv_envelope` rejects expired / skewed / replayed ids
- Durable `peers/envelope_replay.json`; RFC-0092
- QUEUE `#194` **DONE**; first OPEN `#195`

## Out
Run nonce (`#195`); instance-scoped crypto (`#196`); requiring `expires_at` on every sender.
