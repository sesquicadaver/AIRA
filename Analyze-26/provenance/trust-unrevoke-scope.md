# Provenance — Admin Trust Unrevoke

**Decision:** `unrevoke` clears CRL only; never restores `entries` or process keys from `RevokedEntry.public_key_hex`.

**Why:** Operators need an escape hatch after Analyze-25 without silent re-trust of compromised material.

**Non-goals:** Auto re-add, rotation supersedes, audit trail persistence.
