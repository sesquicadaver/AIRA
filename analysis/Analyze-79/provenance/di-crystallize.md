# DI crystallize — Analyze-79 / QUEUE #44

## In
1. Remove `or_else` domain-message verify on `ProtocolEnvelope::validate_signature`.
2. Sign EP/AP envelopes over `payload_hash` bytes so admission still passes.
3. Identity `local_user` signs and verifies `identity_id` bytes (not domain).
4. Prove domain-signed envelopes fail `validate_signature`.

## Out
Canonical Event/Artifact/Object/CSU verify paths; Discovery capability Ed25519 (register only rejects empty); ProtocolResponse has no verify API.
