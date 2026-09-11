# Analyze-364 — activate_verified hash continuity

**QUEUE:** `#327` · **RFC-0212**

## Outcome

`activate_verified` compares source and post-copy SHA-256 to `VerifiedPointer.content_hash`. Mismatch → `ActivateHashMismatch`, no activated pointer.
